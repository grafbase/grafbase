#![cfg_attr(test, allow(unused_crate_dependencies))]

use std::fs;

use args::Args;
use clap::Parser;
use federated_server::Config;
use mimalloc::MiMalloc;
use tokio::runtime;

mod args;
mod global_tracing;

const THREAD_NAME: &str = "grafbase-gateway";

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() -> anyhow::Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("installing default crypto provider");

    let args = Args::parse();
    let config = fs::read_to_string(&args.config)?;
    let mut config: Config = toml::from_str(&config)?;

    global_tracing::init(&mut config, &args)?;

    let runtime = runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name(THREAD_NAME)
        .build()?;

    runtime.block_on(federated_server::start(
        args.listen_address,
        config,
        args.fetch_method()?,
    ))?;

    Ok(())
}
