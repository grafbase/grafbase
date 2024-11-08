use super::FullGraphRef;
use clap::Parser;

/// List published subgraphs
#[derive(Debug, Parser)]
pub struct SubgraphsCommand {
    #[arg(help = FullGraphRef::ARG_DESCRIPTION)]
    pub graph_ref: FullGraphRef,
}
