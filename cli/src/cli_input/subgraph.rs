use super::FullGraphRef;
use clap::{Parser, Subcommand};

/// Manage subgraphs
#[derive(Debug, Parser)]
pub struct SubgraphCommand {
    #[command(subcommand)]
    pub command: SubgraphSubCommand,
}

#[derive(Debug, Subcommand)]
pub enum SubgraphSubCommand {
    /// List all subgraphs
    #[clap(visible_alias = "ls")]
    List {
        #[arg(help = FullGraphRef::ARG_DESCRIPTION)]
        graph_ref: FullGraphRef,
    },
    /// Remove a subgraph
    #[clap(name = "remove", visible_alias = "rm")]
    Delete {
        #[arg(help = FullGraphRef::ARG_DESCRIPTION)]
        graph_ref: FullGraphRef,
        /// Name of the subgraph to delete
        #[arg(help = "The name of the subgraph to delete")]
        name: String,
    },
}
