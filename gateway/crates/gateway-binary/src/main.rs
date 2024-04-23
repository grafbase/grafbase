#![cfg_attr(test, allow(unused_crate_dependencies))]

use ascii as _;
use clap::{crate_version, Parser};
use graph_ref as _;
use mimalloc::MiMalloc;
use tokio::runtime;
use tokio::sync::{oneshot, watch};
use tracing::{debug, error, Subscriber};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{reload, EnvFilter, Layer, Registry};

use args::Args;
use federated_server::{Config, OtelReload, OtelTracing, TelemetryConfig};
use grafbase_tracing::error::TracingError;
use grafbase_tracing::otel::layer::BoxedLayer;
use grafbase_tracing::otel::layer::{self, ReloadableOtelLayers};
use grafbase_tracing::otel::opentelemetry_sdk::runtime::Tokio;
use grafbase_tracing::{otel::opentelemetry_sdk::trace::TracerProvider, span::GRAFBASE_TARGET};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod args;

const THREAD_NAME: &str = "grafbase-gateway";

fn main() -> anyhow::Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("installing default crypto provider");

    let args = Args::parse();
    let mut config = args.config()?;

    let runtime = runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name(THREAD_NAME)
        .build()?;

    runtime.block_on(async move {
        let otel_tracing = setup_tracing(&mut config, &args)?;

        let crate_version = crate_version!();
        tracing::info!(target: GRAFBASE_TARGET, "Grafbase Gateway {crate_version}");

        let listen_address = {
            cfg_if::cfg_if! {
                if #[cfg(feature = "lambda")] {
                    None
                } else {
                    args.listen_address
                }
            }
        };

        federated_server::serve(listen_address, config, args.fetch_method()?, otel_tracing).await?;

        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}

fn setup_tracing(config: &mut Config, args: &Args) -> anyhow::Result<Option<OtelTracing>> {
    // setup tracing globally
    let OtelLegos {
        tracer_provider,
        tracer_layer_reload_handle,
    } = init_global_tracing(args, config.telemetry.clone())?;

    // spawn the otel layer reload
    let (reload_sender, reload_receiver) = oneshot::channel();
    let (tracer_sender, tracer_receiver) = watch::channel(tracer_provider);

    otel_layer_reload(
        reload_receiver,
        tracer_layer_reload_handle,
        tracer_sender,
        config.telemetry.clone(),
    );

    Ok(Some(OtelTracing {
        tracer_provider: tracer_receiver,
        reload_trigger: reload_sender,
    }))
}

struct OtelLegos<S> {
    tracer_provider: TracerProvider,
    tracer_layer_reload_handle: reload::Handle<BoxedLayer<S>, S>,
}

fn init_global_tracing(args: &Args, config: Option<TelemetryConfig>) -> anyhow::Result<OtelLegos<Registry>> {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let filter = args.log_level.map(|l| l.as_filter_str()).unwrap_or("info");

    let env_filter = EnvFilter::new(filter);
    let ReloadableOtelLayers { tracer, meter_provider } = build_otel_layers(config, Default::default())?;
    let tracer = tracer.expect("should have a valid otel trace layer");
    let meter_provider = meter_provider.expect("should have a valid otel trace layer");

    grafbase_tracing::otel::opentelemetry::global::set_tracer_provider(tracer.provider.clone());
    grafbase_tracing::otel::opentelemetry::global::set_meter_provider(meter_provider.clone());

    tracing_subscriber::registry()
        .with(tracer.layer)
        .with(args.log_format())
        .with(env_filter)
        .init();

    Ok(OtelLegos {
        tracer_provider: tracer.provider,
        tracer_layer_reload_handle: tracer.layer_reload_handle,
    })
}

fn otel_layer_reload<S>(
    reload_receiver: oneshot::Receiver<OtelReload>,
    tracer_layer_reload_handle: reload::Handle<BoxedLayer<S>, S>,
    tracer_sender: watch::Sender<TracerProvider>,
    config: Option<TelemetryConfig>,
) where
    S: Subscriber + for<'span> LookupSpan<'span> + Send + Sync,
{
    tokio::spawn(async move {
        let result = reload_receiver.await;

        let Ok(reload_data) = result else {
            debug!("error waiting for otel reload");
            return;
        };

        let ReloadableOtelLayers { tracer, meter_provider } = match build_otel_layers(config, Some(reload_data)) {
            Ok(value) => value,
            Err(err) => {
                error!("error creating a new otel layer for reload: {err}");
                return;
            }
        };
        let tracer = tracer.expect("should have a valid otel trace layer");
        let meter_provider = meter_provider.expect("should have a valid otel trace layer");

        tracer_layer_reload_handle
            .reload(tracer.layer.boxed())
            .expect("should successfully reload otel layer");
        grafbase_tracing::otel::opentelemetry::global::set_tracer_provider(tracer.provider.clone());
        tracer_sender
            .send(tracer.provider)
            .expect("should successfully send new otel tracer");

        grafbase_tracing::otel::opentelemetry::global::set_meter_provider(meter_provider);
    });
}

fn build_otel_layers<S>(
    config: Option<TelemetryConfig>,
    reload_data: Option<OtelReload>,
) -> Result<ReloadableOtelLayers<S>, TracingError>
where
    S: Subscriber + for<'span> LookupSpan<'span> + Send + Sync,
{
    let id_generator = {
        cfg_if::cfg_if! {
            if #[cfg(feature = "lambda")] {
                use opentelemetry_aws::trace::{XrayIdGenerator, XrayPropagator};
                grafbase_tracing::otel::opentelemetry::global::set_text_map_propagator(XrayPropagator::default());

                XrayIdGenerator::default()
            } else {
                use grafbase_tracing::otel::opentelemetry_sdk::trace::RandomIdGenerator;

                RandomIdGenerator::default()
            }
        }
    };

    let config = config.unwrap_or(TelemetryConfig {
        service_name: "unknown".to_string(),
        resource_attributes: Default::default(),
        tracing: Default::default(),
    });

    let mut resource_attributes = config.resource_attributes;
    if let Some(reload_data) = reload_data {
        resource_attributes.insert("grafbase.graph_id".to_string(), reload_data.graph_id.to_string());
        resource_attributes.insert("grafbase.branch_id".to_string(), reload_data.branch_id.to_string());
        resource_attributes.insert("grafbase.branch_name".to_string(), reload_data.branch_name.to_string());
    }
    let resource_attributes = resource_attributes
        .into_iter()
        .map(|(key, value)| grafbase_tracing::otel::opentelemetry::KeyValue::new(key, value))
        .collect::<Vec<_>>();

    layer::new_batched::<S, _, _>(
        config.service_name,
        config.tracing,
        id_generator,
        Tokio,
        resource_attributes,
    )
}
