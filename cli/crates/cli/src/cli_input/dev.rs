use super::ProjectRef;
use clap::Parser;
use std::path::PathBuf;

/// Publish a subgraph
#[derive(Debug, Parser)]
pub struct DevCommand {
    #[arg(help = ProjectRef::ARG_DESCRIPTION)]
    pub(crate) graph_ref: ProjectRef,

    /// The gateway configuration TOML file path
    #[arg(long("gateway-config"))]
    pub(crate) gateway_config: PathBuf,

    /// The graph configuration TOML file path
    #[arg(long("graph-config"))]
    pub(crate) graph_config: PathBuf,
}
