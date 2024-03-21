use std::fs;

use federated_server::Config;
use mimalloc::MiMalloc;
use tokio::runtime;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const THREAD_NAME: &str = "grafbase-gateway-lambda";

fn main() -> anyhow::Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("installing default crypto provider");

    let config: Config = match std::env::var("GRAFBASE_CONFIG_PATH") {
        Ok(path) => {
            let config = fs::read_to_string(path)?;
            toml::from_str(&config)?
        }
        Err(_) => return Err(anyhow::anyhow!("environment variable GRAFBASE_CONFIG_PATH must be set")),
    };

    let federated_schema = match std::env::var("GRAFBASE_SCHEMA_PATH") {
        Ok(path) => fs::read_to_string(path)?,
        Err(_) => return Err(anyhow::anyhow!("environment variable GRAFBASE_SCHEMA_PATH must be set")),
    };

    let runtime = runtime::Builder::new_current_thread()
        .enable_all()
        .thread_name(THREAD_NAME)
        .build()?;

    let fetch_method = federated_server::GraphFetchMethod::FromLocal { federated_schema };

    runtime.block_on(federated_server::start(None, config, fetch_method))?;

    Ok(())
}
