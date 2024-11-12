#![cfg_attr(test, allow(unused_crate_dependencies))]

use grafbase_workspace_hack as _;

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
        let telemetry = telemetry::init(&args, &config.telemetry)?;

        let crate_version = crate_version!();
        tracing::info!("Grafbase Gateway {crate_version}");

        let config = ServerConfig {
            listen_addr: args.listen_address(),
            config,
            config_path: args.config_path().map(|p| p.to_owned()),
            config_hot_reload: args.hot_reload(),
            fetch_method: args.fetch_method()?,
        };

        let server_runtime = server_runtime::build(telemetry.clone());

        let result = federated_server::serve(config, server_runtime)
            .await
            .map_err(anyhow::Error::from);

        telemetry.graceful_shutdown().await;

        result
    })
}
