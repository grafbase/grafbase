use std::path::PathBuf;

use clap::Parser;

use super::FullGraphRef;

/// Compose a federated schema.
#[derive(Debug, Parser)]
pub(crate) struct ComposeCommand {
    #[arg(short('r'), long("graph-ref"), help = FullGraphRef::ARG_DESCRIPTION)]
    pub(crate) graph_ref: Option<FullGraphRef>,
    /// The path of the gateway configuration file
    #[arg(short('c'), long("gateway-config"))]
    pub(crate) gateway_config: Option<PathBuf>,
    /// The path of the graph overrides configuration file
    #[arg(short('o'), long("graph-overrides"))]
    pub(crate) graph_overrides: Option<PathBuf>,
}
