use std::path::PathBuf;

use clap::Parser;

use super::FullGraphRef;

/// Compose a federated schema.
#[derive(Debug, Parser)]
pub(crate) struct ComposeCommand {
    #[arg(short('r'), long("graph-ref"), help = FullGraphRef::ARG_DESCRIPTION)]
    pub(crate) graph_ref: Option<FullGraphRef>,
    /// The path of the gateway configuration file
    #[arg(short('c'), long("config"))]
    pub(crate) config_path: PathBuf,
}
