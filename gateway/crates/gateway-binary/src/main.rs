#![cfg_attr(test, allow(unused_crate_dependencies))]

use args::Args;
use clap::Parser;
use grafbase_tracing::otel::layer::BoxedLayer;
use mimalloc::MiMalloc;
use tracing_core::Subscriber;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;
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
    let (otel_layer, reload_handle) = grafbase_tracing::otel::layer::new_noop();

    tracing_subscriber::registry()
        .with(Some(otel_layer))
        .with(log_format_layer())
        .with(filter)
        .init();

    federated_server::start(args.listen_address, &args.config, args.fetch_method()?, reload_handle)?;

    Ok(())
}

#[cfg(not(feature = "lambda"))]
fn log_format_layer<S>() -> BoxedLayer<S>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    tracing_subscriber::fmt::layer().boxed()
}

#[cfg(feature = "lambda")]
fn log_format_layer<S>() -> BoxedLayer<S>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    tracing_subscriber::fmt::layer().json().boxed()
}
