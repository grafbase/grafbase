use args::Args;
use clap::crate_version;
use mimalloc::MiMalloc;
use tokio::{runtime, sync::watch};

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

        if !args.can_export_telemetry_to_platform() {
            tracing::warn!("To send telemetry to the Grafbase Platform, provide a valid graph-ref and access token");
        }

        let config = ServerConfig {
            listen_addr: args.listen_address(),
            config_receiver: config_recevier(config),
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

fn config_recevier(config: gateway_config::Config) -> watch::Receiver<gateway_config::Config> {
    let (sender, receiver) = watch::channel(config);

    // Leak the sender so the channel never closes
    //
    // This should be safe since this function is only ever called once from fn main()
    Box::leak(Box::new(sender));

    receiver
}
