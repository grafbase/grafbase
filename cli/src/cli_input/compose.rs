use std::path::PathBuf;

use clap::Parser;
use gateway_config::Config;

use super::FullGraphRef;

/// Compose a federated schema.
#[derive(Debug, Parser)]
pub(crate) struct ComposeCommand {
    #[arg(short('r'), long("graph-ref"), help = FullGraphRef::ARG_DESCRIPTION)]
    pub(crate) graph_ref: Option<FullGraphRef>,
    /// The path of the gateway configuration file
    #[arg(short('c'), long("config"))]
    config_path: Option<PathBuf>,
}

impl ComposeCommand {
    pub fn config(&self) -> anyhow::Result<Config> {
        Config::loader()
            .load_or_default(self.config_path.as_ref())
            .map_err(|err| anyhow::anyhow!(err))
    }
}
