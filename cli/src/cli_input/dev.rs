use super::FullGraphRef;
use clap::Parser;
use gateway_config::Config;
use std::{net::SocketAddr, path::PathBuf};

/// Start a local development server
#[derive(Debug, Parser)]
pub struct DevCommand {
    #[arg(short('g'), long("graph-ref"), help = FullGraphRef::ARG_DESCRIPTION)]
    pub(crate) graph_ref: Option<FullGraphRef>,
    /// The path of the gateway configuration file
    #[arg(short('c'), long("config"))]
    config_path: Option<PathBuf>,
    /// The port to listen on for requests
    #[arg(short('p'), long("port"))]
    pub(crate) port: Option<u16>,
    /// Listen address for the server, overrides the port
    #[arg(long)]
    pub listen_address: Option<SocketAddr>,
}

impl DevCommand {
    pub fn config(&self) -> anyhow::Result<Config> {
        Config::loader()
            .load(self.config_path.as_ref())
            .map(Option::unwrap_or_default)
            .map_err(|err| anyhow::anyhow!(err))
    }
}
