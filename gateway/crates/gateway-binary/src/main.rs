#![cfg_attr(test, allow(unused_crate_dependencies))]

use args::Args;
use ascii as _;
use clap::crate_version;
use gateway_config::Config;
use graph_ref as _;
use mimalloc::MiMalloc;
use tokio::runtime;
use tokio::sync::{oneshot, watch};
use tracing::{error, Subscriber};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{reload, EnvFilter, Layer, Registry};

use federated_server::{GraphFetchMethod, OtelReload, OtelTracing, ServerConfig};
use grafbase_telemetry::config::TelemetryConfig;
use grafbase_telemetry::error::TracingError;
use grafbase_telemetry::otel::layer::BoxedLayer;
use grafbase_telemetry::otel::layer::{self, ReloadableOtelLayers};
use grafbase_telemetry::otel::opentelemetry_sdk::runtime::Tokio;
use grafbase_telemetry::{otel::opentelemetry_sdk::trace::TracerProvider, span::GRAFBASE_TARGET};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod args;

const THREAD_NAME: &str = "grafbase-gateway";

fn main() -> anyhow::Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("installing default crypto provider");

    let args = self::args::parse();
    let mut config = args.config()?;

    let runtime = runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name(THREAD_NAME)
        .build()?;

    runtime.block_on(async move {
        let otel_tracing = if std::env::var("__GRAFBASE_RUST_LOG").is_ok() {
            let filter = tracing_subscriber::filter::EnvFilter::try_from_env("__GRAFBASE_RUST_LOG").unwrap_or_default();

            tracing_subscriber::fmt()
                .pretty()
                .with_env_filter(filter)
                .with_file(true)
                .with_line_number(true)
                .with_target(true)
                .without_time()
                .init();

            tracing::warn!("Skipping OTEL configuration.");

            None
        } else {
            setup_tracing(&mut config, &args)?
        };

        let crate_version = crate_version!();
        tracing::info!(target: GRAFBASE_TARGET, "Grafbase Gateway {crate_version}");

        federated_server::serve(ServerConfig {
            listen_addr: args.listen_address(),
            config,
            config_path: args.config_path().map(|p| p.to_owned()),
            config_hot_reload: args.hot_reload(),
            fetch_method: args.fetch_method()?,
            otel_tracing,
        })
        .await?;

        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}

fn setup_tracing(config: &mut Config, args: &impl Args) -> anyhow::Result<Option<OtelTracing>> {
    // setup tracing globally
    let OtelLegos {
        tracer_provider,
        tracer_layer_reload_handle,
    } = init_global_tracing(args, config.telemetry.clone())?;

    // spawn the otel layer reload
    let (reload_sender, reload_receiver) = oneshot::channel();
    let (reload_ack_sender, reload_ack_receiver) = oneshot::channel();
    let (tracer_sender, tracer_receiver) = watch::channel(tracer_provider);

    otel_layer_reload(
        reload_receiver,
        reload_ack_sender,
        tracer_layer_reload_handle,
        tracer_sender,
        config.telemetry.clone(),
    );

    Ok(Some(OtelTracing {
        tracer_provider: tracer_receiver,
        reload_trigger: reload_sender,
        reload_ack_receiver,
    }))
}

struct OtelLegos<S> {
    tracer_provider: TracerProvider,
    tracer_layer_reload_handle: reload::Handle<BoxedLayer<S>, S>,
}

fn init_global_tracing(args: &impl Args, config: Option<TelemetryConfig>) -> anyhow::Result<OtelLegos<Registry>> {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let filter = args.log_level().map(|l| l.as_filter_str()).unwrap_or("info");
    let env_filter = EnvFilter::new(filter);
    let will_reload_otel = !matches!(args.fetch_method()?, GraphFetchMethod::FromLocal { .. });

    let ReloadableOtelLayers {
        tracer,
        meter_provider,
        logger,
    } = build_otel_layers(config, Default::default(), will_reload_otel)?;

    let tracer = tracer.expect("should have a valid otel trace layer");

    grafbase_telemetry::otel::opentelemetry::global::set_tracer_provider(tracer.provider.clone());

    if let Some(meter_provider) = meter_provider {
        grafbase_telemetry::otel::opentelemetry::global::set_meter_provider(meter_provider);
    }

    match logger {
        Some(logger) => {
            tracing_subscriber::registry()
                .with(tracer.layer.boxed())
                .with(logger.boxed())
                .with(args.log_format())
                .with(env_filter)
                .init();
        }
        None => {
            tracing_subscriber::registry()
                .with(tracer.layer)
                .with(args.log_format())
                .with(env_filter)
                .init();
        }
    }

    Ok(OtelLegos {
        tracer_provider: tracer.provider,
        tracer_layer_reload_handle: tracer.layer_reload_handle,
    })
}

fn otel_layer_reload<S>(
    reload_receiver: oneshot::Receiver<OtelReload>,
    reload_ack_sender: oneshot::Sender<()>,
    tracer_layer_reload_handle: reload::Handle<BoxedLayer<S>, S>,
    tracer_sender: watch::Sender<TracerProvider>,
    config: Option<TelemetryConfig>,
) where
    S: Subscriber + for<'span> LookupSpan<'span> + Send + Sync,
{
    tokio::spawn(async move {
        let result = reload_receiver.await;
        tracing::trace!("Initializing Grafbase telemetry");

        let Ok(reload_data) = result else {
            tracing::debug!("Grafbase telemetry is disabled.");
            reload_ack_sender.send(()).ok();
            return;
        };

        let ReloadableOtelLayers {
            tracer,
            meter_provider,
            logger: _logger,
        } = match build_otel_layers(config, Some(reload_data), false) {
            Ok(value) => value,
            Err(err) => {
                error!("error creating a new otel layer for reload: {err}");
                reload_ack_sender.send(()).ok();
                return;
            }
        };

        let Some(tracer) = tracer else {
            error!("should have a valid otel trace layer");
            reload_ack_sender.send(()).ok();
            return;
        };

        let Some(meter_provider) = meter_provider else {
            error!("should have a valid otel meter provider");
            reload_ack_sender.send(()).ok();
            return;
        };

        if let Err(err) = tracer_layer_reload_handle.reload(tracer.layer.boxed()) {
            error!("error reloading otel layer: {err}");
            reload_ack_sender.send(()).ok();
            return;
        }

        grafbase_telemetry::otel::opentelemetry::global::set_meter_provider(meter_provider);
        grafbase_telemetry::otel::opentelemetry::global::set_tracer_provider(tracer.provider.clone());

        reload_ack_sender.send(()).ok();

        // FIXME: this seems to block the reload, but it's not clear why
        tracer_sender.send(tracer.provider).ok();
    });
}

fn build_otel_layers<S>(
    config: Option<TelemetryConfig>,
    reload_data: Option<OtelReload>,
    will_reload_otel: bool,
) -> Result<ReloadableOtelLayers<S>, TracingError>
where
    S: Subscriber + for<'span> LookupSpan<'span> + Send + Sync,
{
    let id_generator = {
        cfg_if::cfg_if! {
            if #[cfg(feature = "lambda")] {
                use opentelemetry_aws::trace::{XrayIdGenerator, XrayPropagator};
                grafbase_telemetry::otel::opentelemetry::global::set_text_map_propagator(XrayPropagator::default());

                XrayIdGenerator::default()
            } else {
                use grafbase_telemetry::otel::opentelemetry_sdk::trace::RandomIdGenerator;

                RandomIdGenerator::default()
            }
        }
    };

    let mut config = config.unwrap_or(TelemetryConfig {
        service_name: "unknown".to_string(),
        resource_attributes: Default::default(),
        tracing: Default::default(),
        exporters: Default::default(),
        logs: Default::default(),
        metrics: Default::default(),
        grafbase: Default::default(),
    });

    if let Some(reload_data) = reload_data {
        config
            .resource_attributes
            .insert("grafbase.graph_id".to_string(), reload_data.graph_id.to_string());
        config
            .resource_attributes
            .insert("grafbase.branch_id".to_string(), reload_data.branch_id.to_string());
        config
            .resource_attributes
            .insert("grafbase.branch_name".to_string(), reload_data.branch_name.to_string());
    }

    layer::new_batched::<S, _, _>(config, id_generator, Tokio, will_reload_otel)
}
