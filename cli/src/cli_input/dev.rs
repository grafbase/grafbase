use super::FullGraphRef;
use clap::Parser;
use gateway_config::Config;
use std::path::PathBuf;

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
}

impl DevCommand {
    pub fn config(&self) -> anyhow::Result<Config> {
        Config::loader()
            .load_or_default(self.config_path.as_ref())
            .map_err(|err| anyhow::anyhow!(err))
    }
}
