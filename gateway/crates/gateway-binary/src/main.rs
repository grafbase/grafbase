#![cfg_attr(test, allow(unused_crate_dependencies))]

use std::fs;

use args::Args;
use clap::Parser;
use federated_server::Config;
use grafbase_tracing as _;
use mimalloc::MiMalloc;
use tokio::runtime;
use tracing_subscriber::EnvFilter;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod args;

const THREAD_NAME: &str = "grafbase-gateway";

fn main() -> anyhow::Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("installing default crypto provider");

    let args = Args::parse();
    let config = fs::read_to_string(&args.config)?;
    let mut config: Config = toml::from_str(&config)?;

    let filter = EnvFilter::builder().parse_lossy(args.log_filter());

    let runtime = runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name(THREAD_NAME)
        .build()?;

    runtime.block_on(async move {
        // if this is not called from tokio context, you'll get:
        // there is no reactor running, must be called from the context of a Tokio 1.x runtime
        // but... only if telemetry is enabled, so be aware and read this when you have a failing test!
        init_global_tracing(filter, &mut config)?;

        federated_server::start(args.listen_address, config, args.fetch_method()?).await?;

        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}

#[cfg(not(feature = "lambda"))]
fn init_global_tracing(filter: EnvFilter, config: &mut Config) -> anyhow::Result<()> {
    use grafbase_tracing::otel::{layer, opentelemetry_sdk::runtime::Tokio};
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

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

    Ok(())
}

#[cfg(feature = "lambda")]
fn init_global_tracing(_: EnvFilter, _: &mut Config) -> anyhow::Result<()> {
    Ok(())
}
