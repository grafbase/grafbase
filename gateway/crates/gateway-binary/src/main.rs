#![cfg_attr(test, allow(unused_crate_dependencies))]

use std::fs;

use anyhow::Context;
use clap::{crate_version, Parser};
use mimalloc::MiMalloc;
use tokio::runtime;
use tokio::sync::{oneshot, watch};
use tracing::{debug, error, Subscriber};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{reload, EnvFilter, Registry};

use args::Args;
use federated_server::{Config, OtelReload, OtelTracing, TelemetryConfig};
use grafbase_tracing::error::TracingError;
use grafbase_tracing::otel::layer;
use grafbase_tracing::otel::layer::{BoxedLayer, ReloadableLayer};
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
    let config = fs::read_to_string(&args.config).context("could not read config file")?;
    let mut config: Config = toml::from_str(&config)?;

    let runtime = runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name(THREAD_NAME)
        .build()?;

    runtime.block_on(async move {
        let otel_tracing = setup_tracing(&mut config, &args)?;

        let crate_version = crate_version!();
        tracing::info!(target: GRAFBASE_TARGET, "Grafbase Gateway {crate_version}");

        federated_server::serve(args.listen_address, config, args.fetch_method()?, otel_tracing).await?;

        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}

fn setup_tracing(config: &mut Config, args: &Args) -> anyhow::Result<Option<OtelTracing>> {
    let telemetry_config = match config.telemetry.take() {
        Some(telemetry_config) if telemetry_config.tracing.enabled => telemetry_config,
        _ => return Ok(None),
    };

    // setup tracing globally
    let OtelLegos {
        provider,
        reload_handle,
    } = init_global_tracing(args, telemetry_config.clone())?;

    grafbase_tracing::otel::opentelemetry::global::set_tracer_provider(provider.clone());

    // spawn the otel layer reload
    let (reload_sender, reload_receiver) = oneshot::channel();
    let (tracer_sender, tracer_receiver) = watch::channel(provider);

    otel_layer_reload(reload_handle, reload_receiver, tracer_sender, telemetry_config);

    Ok(Some(OtelTracing {
        tracer_provider: tracer_receiver,
        reload_trigger: reload_sender,
    }))
}

struct OtelLegos<S> {
    provider: TracerProvider,
    reload_handle: reload::Handle<BoxedLayer<S>, S>,
}

fn init_global_tracing(args: &Args, config: TelemetryConfig) -> anyhow::Result<OtelLegos<Registry>> {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let filter = args
        .log_level
        .map(|l| l.as_filter_str())
        .unwrap_or(config.tracing.filter.as_str());

    let env_filter = EnvFilter::new(filter);
    let otel_layer = build_otel_layer(config, Default::default())?;

    tracing_subscriber::registry()
        .with(otel_layer.layer)
        .with(args.log_format())
        .with(env_filter)
        .init();

    Ok(OtelLegos {
        provider: otel_layer.provider.expect("should have a valid otel tracer provider"),
        reload_handle: otel_layer.handle,
    })
}

fn otel_layer_reload<S>(
    reload_handle: reload::Handle<BoxedLayer<S>, S>,
    reload_receiver: oneshot::Receiver<OtelReload>,
    tracer_sender: watch::Sender<TracerProvider>,
    config: TelemetryConfig,
) where
    S: Subscriber + for<'span> LookupSpan<'span> + Send + Sync,
{
    use tracing_subscriber::Layer;

    tokio::spawn(async move {
        let result = reload_receiver.await;

        let Ok(reload_data) = result else {
            debug!("error waiting for otel reload");
            return;
        };

        let otel_layer = match build_otel_layer(config, reload_data) {
            Ok(value) => value,
            Err(err) => {
                error!("error creating a new otel layer for reload: {err}");
                return;
            }
        };

        reload_handle
            .reload(otel_layer.layer.boxed())
            .expect("should successfully reload otel layer");

        let tracer_provider = otel_layer
            .provider
            .expect("should have a new valid otel tracer provder");

        grafbase_tracing::otel::opentelemetry::global::set_tracer_provider(tracer_provider.clone());

        tracer_sender
            .send(tracer_provider)
            .expect("should successfully send new otel tracer");
    });
}

fn build_otel_layer<S>(config: TelemetryConfig, reload_data: OtelReload) -> Result<ReloadableLayer<S>, TracingError>
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

    let resource_attributes = vec![
        grafbase_tracing::otel::opentelemetry::KeyValue::new("graph_id", u128::from(reload_data.graph_id).to_string()),
        grafbase_tracing::otel::opentelemetry::KeyValue::new(
            "branch_id",
            u128::from(reload_data.branch_id).to_string(),
        ),
        grafbase_tracing::otel::opentelemetry::KeyValue::new("branch_name", reload_data.branch_name.to_string()),
    ];

    layer::new_batched::<S, _, _>(
        config.service_name,
        config.tracing,
        id_generator,
        Tokio,
        resource_attributes,
    )
}
