#![cfg_attr(test, allow(unused_crate_dependencies))]

use args::Args;
use clap::Parser;
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

    let filter = EnvFilter::builder().parse_lossy(args.log_filter());

    start_server(filter, args)?;

    Ok(())
}

#[cfg(not(feature = "lambda"))]
fn start_server(filter: EnvFilter, args: Args) -> Result<(), anyhow::Error> {
    let (otel_layer, reload_handle) = grafbase_tracing::otel::layer::new_noop();

    tracing_subscriber::registry()
        .with(Some(otel_layer))
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();

    federated_server::start(args.listen_address, &args.config, args.fetch_method()?, reload_handle)?;

    Ok(())
}

#[cfg(feature = "lambda")]
fn start_server(filter: EnvFilter, args: Args) -> Result<(), anyhow::Error> {
    use grafbase_tracing::otel::opentelemetry::global;
    use opentelemetry_aws::trace::XrayPropagator;

    global::set_text_map_propagator(XrayPropagator::default());

    let (otel_layer, reload_handle) = grafbase_tracing::otel::layer::new_noop();

    tracing_subscriber::registry()
        .with(Some(otel_layer))
        .with(tracing_subscriber::fmt::layer().json())
        .with(filter)
        .init();

    federated_server::start(args.listen_address, &args.config, args.fetch_method()?, reload_handle)?;

    Ok(())
}
