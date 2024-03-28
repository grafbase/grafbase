#![cfg_attr(test, allow(unused_crate_dependencies))]

use std::fs;

use anyhow::Context;
use args::Args;
use clap::{crate_version, Parser};
use federated_server::Config;
use grafbase_tracing::{otel::opentelemetry_sdk::trace::TracerProvider, span::GRAFBASE_TARGET};
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
    let config = fs::read_to_string(&args.config).context("could not read config file")?;
    let mut config: Config = toml::from_str(&config)?;

    let runtime = runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name(THREAD_NAME)
        .build()?;

    runtime.block_on(async move {
        // if this is not called from tokio context, you'll get:
        // there is no reactor running, must be called from the context of a Tokio 1.x runtime
        // but... only if telemetry is enabled, so be aware and read this when you have a failing test!
        let provider = init_global_tracing(&args, &mut config)?;

        let crate_version = crate_version!();
        tracing::info!(target: GRAFBASE_TARGET, "Grafbase Gateway {crate_version}");

        federated_server::serve(args.listen_address, config, args.fetch_method()?, provider).await?;

        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}

#[cfg(not(feature = "lambda"))]
fn init_global_tracing(args: &Args, config: &mut Config) -> anyhow::Result<Option<TracerProvider>> {
    use grafbase_tracing::otel::{layer, opentelemetry_sdk::runtime::Tokio};
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let (otel_layer, filter) = match config.telemetry.take() {
        Some(config) => {
            let filter = args
                .log_level
                .map(|l| l.as_filter_str())
                .unwrap_or(config.tracing.filter.as_str());

            let env_filter = EnvFilter::new(filter);
            let otel_layer = layer::new_batched(&config.service_name, config.tracing, Tokio)?;

            (Some(otel_layer), env_filter)
        }
        None => (None, EnvFilter::new(args.log_level.unwrap_or_default().as_filter_str())),
    };

    tracing_subscriber::registry()
        .with(otel_layer)
        .with(args.log_format())
        .with(filter)
        .init();

    Ok(None)
}

#[cfg(feature = "lambda")]
fn init_global_tracing(args: &Args, config: &mut Config) -> anyhow::Result<Option<TracerProvider>> {
    use grafbase_tracing::otel::opentelemetry::global;
    use grafbase_tracing::otel::opentelemetry::trace::TracerProvider;
    use grafbase_tracing::otel::tracing_opentelemetry;
    use grafbase_tracing::otel::tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    use grafbase_tracing::otel::{self, opentelemetry_sdk::runtime::Tokio};
    use opentelemetry_aws::trace::{XrayIdGenerator, XrayPropagator};

    global::set_text_map_propagator(XrayPropagator::default());

    let (provider, filter) = match config.telemetry.take() {
        Some(config) => {
            let filter = args
                .log_level
                .map(|l| l.as_filter_str())
                .unwrap_or(config.tracing.filter.as_str());

            let filter = EnvFilter::new(filter);

            let provider =
                otel::provider::create(&config.service_name, config.tracing, XrayIdGenerator::default(), Tokio)
                    .expect("error creating otel provider");

            (Some(provider), filter)
        }
        None => (None, EnvFilter::new(args.log_level.unwrap_or_default().as_filter_str())),
    };

    let subscriber = tracing_subscriber::registry();

    match provider {
        Some(ref provider) => {
            let tracer = provider.tracer("lambda-otel");

            subscriber
                .with(tracing_opentelemetry::layer().with_tracer(tracer))
                .with(args.log_format())
                .with(filter)
                .init();
        }
        None => {
            subscriber.with(args.log_format()).with(filter).init();
        }
    }

    Ok(provider)
}
