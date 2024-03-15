#![cfg_attr(test, allow(unused_crate_dependencies))]

use args::Args;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod args;

fn main() -> anyhow::Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("installing default crypto provider");

    let args = Args::parse();

    let filter = EnvFilter::builder().parse_lossy(args.log_filter());
    let (otel_layer, reload_handle) = grafbase_tracing::otel::layer::new_noop();

    tracing_subscriber::registry()
        .with(Some(otel_layer))
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();

    federated_server::start(args.listen_address, &args.config, args.fetch_method()?, reload_handle)?;

    Ok(())
}
