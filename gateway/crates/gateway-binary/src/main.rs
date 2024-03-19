#![cfg_attr(test, allow(unused_crate_dependencies))]

use std::fs;

use args::Args;
use clap::Parser;
use federated_server::Config;
use mimalloc::MiMalloc;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod args;

fn main() -> anyhow::Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("installing default crypto provider");

    let args = Args::parse();
    let config = fs::read_to_string(&args.config)?;
    let config: Config = toml::from_str(&config)?;

    let filter = EnvFilter::builder().parse_lossy(args.log_filter());

    start_server(filter, args, config)?;

    Ok(())
}

#[cfg(not(feature = "lambda"))]
fn start_server(filter: EnvFilter, args: Args, mut config: Config) -> Result<(), anyhow::Error> {
    // let (otel_layer, reload_handle) = grafbase_tracing::otel::layer::new_noop();

    use grafbase_tracing::otel::{layer, opentelemetry_sdk::runtime::Tokio};

    let (otel_layer, filter) = match config.telemetry.take() {
        Some(config) => {
            let env_filter = EnvFilter::new(&config.tracing.filter);
            let otel_layer = layer::new_batched(&config.service_name, config.tracing, Tokio)?;

            (Some(otel_layer), env_filter)
        }
        None => (None, filter),
    };

    tracing_subscriber::registry()
        .with(otel_layer)
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();

    federated_server::start(args.listen_address, config, args.fetch_method()?)?;

    Ok(())
}

#[cfg(feature = "lambda")]
fn start_server(filter: EnvFilter, args: Args, config: Config) -> Result<(), anyhow::Error> {
    use grafbase_tracing::otel::opentelemetry::global;
    use grafbase_tracing::otel::opentelemetry::trace::TracerProvider as _;
    use grafbase_tracing::otel::opentelemetry_sdk::trace::TracerProvider;
    use opentelemetry_aws::trace::XrayPropagator;

    global::set_text_map_propagator(XrayPropagator::default());

    let filter = config
        .telemetry
        .as_ref()
        .map(|config| EnvFilter::new(&config.tracing.filter))
        .unwrap_or(filter);

    let otel_layer = match config
        .telemetry
        .as_ref()
        .and_then(|config| config.tracing.exporters.stdout.as_ref())
    {
        Some(stdout_config) if stdout_config.enabled => {
            let otel_service_name = config
                .telemetry
                .as_ref()
                .map(|config| config.service_name.as_str())
                .unwrap_or("grafbase-gateway");

            let provider = TracerProvider::builder()
                .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
                .build();

            let tracer = provider.tracer(otel_service_name.to_string());
            let otel_layer = grafbase_tracing::otel::tracing_opentelemetry::layer().with_tracer(tracer);

            Some(otel_layer)
        }
        _ => None,
    };

    tracing_subscriber::registry()
        .with(otel_layer)
        .with(filter)
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    federated_server::start(args.listen_address, config, args.fetch_method()?)?;

    Ok(())
}
