#![cfg_attr(test, allow(unused_crate_dependencies))]

use args::Args;
use clap::Parser;
use federated_server::Config;
use grafbase_tracing::otel::opentelemetry_sdk::trace::TracerProvider;
use mimalloc::MiMalloc;
use tokio::runtime;
use tracing_subscriber::util::SubscriberInitExt;
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
    let license = args.license()?;
    let mut config = args.config(&license)?;

    let filter = EnvFilter::builder().parse_lossy(args.log_filter());

    let runtime = runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name(THREAD_NAME)
        .build()?;

    runtime.block_on(async move {
        // if this is not called from tokio context, you'll get:
        // there is no reactor running, must be called from the context of a Tokio 1.x runtime
        // but... only if telemetry is enabled, so be aware and read this when you have a failing test!
        let provider = init_global_tracing(filter, &mut config)?;

        federated_server::start(args.listen_address, config, args.fetch_method(license)?, provider).await?;

        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}

#[cfg(not(feature = "lambda"))]
fn init_global_tracing(filter: EnvFilter, config: &mut Config) -> anyhow::Result<Option<TracerProvider>> {
    use grafbase_tracing::otel::{layer, opentelemetry_sdk::runtime::Tokio};
    use tracing_subscriber::layer::SubscriberExt;

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

    Ok(None)
}

#[cfg(feature = "lambda")]
fn init_global_tracing(filter: EnvFilter, config: &mut Config) -> anyhow::Result<Option<TracerProvider>> {
    use grafbase_tracing::otel::opentelemetry::global;
    use grafbase_tracing::otel::opentelemetry::trace::TracerProvider;
    use grafbase_tracing::otel::tracing_opentelemetry;
    use grafbase_tracing::otel::tracing_subscriber::layer::SubscriberExt;
    use grafbase_tracing::otel::{self, opentelemetry_sdk::runtime::Tokio};
    use opentelemetry_aws::trace::{XrayIdGenerator, XrayPropagator};

    global::set_text_map_propagator(XrayPropagator::default());

    let (provider, filter) = match config.telemetry.take() {
        Some(config) => {
            let filter = EnvFilter::new(&config.tracing.filter);

            let provider =
                otel::provider::create(&config.service_name, config.tracing, XrayIdGenerator::default(), Tokio)
                    .expect("error creating otel provider");

            (Some(provider), filter)
        }
        None => (None, filter),
    };

    let subscriber = tracing_subscriber::registry();

    match provider {
        Some(ref provider) => {
            let tracer = provider.tracer("lambda-otel");

            subscriber
                .with(tracing_opentelemetry::layer().with_tracer(tracer))
                .with(tracing_subscriber::fmt::layer().with_ansi(false))
                .with(filter)
                .init();
        }
        None => {
            subscriber
                .with(tracing_subscriber::fmt::layer().with_ansi(false))
                .with(filter)
                .init();
        }
    }

    Ok(provider)
}
