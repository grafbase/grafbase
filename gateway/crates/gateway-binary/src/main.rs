#![cfg_attr(test, allow(unused_crate_dependencies))]

use args::Args;
use clap::crate_version;
use mimalloc::MiMalloc;
use tokio::runtime;

use federated_server::ServerConfig;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod args;
mod server_runtime;
mod telemetry;

const THREAD_NAME: &str = "grafbase-gateway";

fn main() -> anyhow::Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("installing default crypto provider");

    let args = self::args::parse();
    let config = args.config()?;

    let runtime = runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name(THREAD_NAME)
        .build()?;

    runtime.block_on(async move {
        let telemetry = if std::env::var("__GRAFBASE_RUST_LOG").is_ok() {
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
            Default::default()
        } else {
            telemetry::init(&args, config.telemetry.clone())?
        };

        let crate_version = crate_version!();
        tracing::info!("Grafbase Gateway {crate_version}");

        let config = ServerConfig {
            listen_addr: args.listen_address(),
            config,
            config_path: args.config_path().map(|p| p.to_owned()),
            config_hot_reload: args.hot_reload(),
            fetch_method: args.fetch_method()?,
        };
        let runtime = server_runtime::build(telemetry);

        federated_server::serve(config, runtime).await?;

        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}
