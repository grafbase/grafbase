use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use args::Args;
use clap::crate_version;
use tokio::{runtime, sync::watch};

use federated_server::ServeConfig;

mod args;
mod config;
mod server_runtime;
mod telemetry;

const THREAD_NAME: &str = "grafbase-gateway";

fn main() -> anyhow::Result<()> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("installing default crypto provider");

    let args = self::args::parse();
    let config = self::config::load(&args)?;

    let runtime = runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name(THREAD_NAME)
        .build()?;

    runtime.block_on(async move {
        let telemetry = telemetry::init(&args, &config.telemetry)?;

        for (name, subgraph) in &config.subgraphs {
            if subgraph.introspection_url.is_some()
                || subgraph.introspection_headers.is_some()
                || subgraph.schema_path.is_some()
            {
                tracing::warn!(
                    "Subgraph {name} has introspection_url, introspection_headers or schema_path set. They're ignored in the federated gateway."
                );
            }
        }

        let crate_version = crate_version!();
        tracing::info!("Grafbase Gateway {crate_version}");

        if !args.can_export_telemetry_to_platform() {
            tracing::warn!("To send telemetry to the Grafbase Platform, provide a valid graph-ref and access token");
        }

        let config_receiver = config_receiver(config);

        const DEFAULT_LISTEN_ADDRESS: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5000);
        let listen_address = args
            .listen_address()
            .or(config_receiver.borrow().network.listen_address)
            .unwrap_or(DEFAULT_LISTEN_ADDRESS);

        let logging_filter = args.log_level().to_string();

        let config = ServeConfig {
            listen_address,
            config_receiver,
            config_path: args.config_path().map(|p| p.to_owned()),
            config_hot_reload: args.hot_reload(),
            graph_loader: args.fetch_method()?,
            grafbase_access_token: args.grafbase_access_token()?,
            logging_filter,
        };

        let server_runtime = server_runtime::build(telemetry.clone());

        let result = federated_server::serve(config, server_runtime)
            .await
            .map_err(anyhow::Error::from);

        telemetry.graceful_shutdown().await;

        result
    })
}

fn config_receiver(config: gateway_config::Config) -> watch::Receiver<gateway_config::Config> {
    let (sender, receiver) = watch::channel(config);

    // Leak the sender so the channel never closes
    //
    // This should be safe since this function is only ever called once from fn main()
    Box::leak(Box::new(sender));

    receiver
}
