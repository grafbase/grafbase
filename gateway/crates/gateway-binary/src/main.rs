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
fn start_server(filter: EnvFilter, args: Args, mut config: Config) -> Result<(), anyhow::Error> {
    use grafbase_tracing::otel::layer;
    use grafbase_tracing::otel::opentelemetry::global;
    use grafbase_tracing::otel::opentelemetry_sdk::runtime::Tokio;
    use opentelemetry_aws::trace::XrayPropagator;

    global::set_text_map_propagator(XrayPropagator::default());

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
        .with(filter)
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    federated_server::start(args.listen_address, config, args.fetch_method()?)?;

    Ok(())
}
