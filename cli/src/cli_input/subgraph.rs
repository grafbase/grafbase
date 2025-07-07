use super::FullGraphRef;
use clap::Parser;

/// Manage subgraphs
#[derive(Debug, Parser)]
pub struct SubgraphCommand {
    #[command(subcommand)]
    pub command: SubgraphSubCommand,
}

#[derive(Debug, Parser, strum::AsRefStr, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum SubgraphSubCommand {
    /// List all subgraphs
    #[clap(visible_alias = "ls")]
    List(SubgraphListCommand),
    #[clap(name = "remove", visible_alias = "rm")]
    Delete(SubgraphDeleteCommand),
}

#[derive(Debug, Parser)]
pub struct SubgraphListCommand {
    /// Graph ref
    #[arg(help = FullGraphRef::ARG_DESCRIPTION)]
    pub graph_ref: FullGraphRef,
}

#[derive(Debug, Parser)]
pub struct SubgraphDeleteCommand {
    /// Branch ref
    #[arg(help = FullGraphRef::ARG_DESCRIPTION)]
    pub graph_ref: FullGraphRef,
    /// Name of the subgraph to delete
    #[arg(help = "The name of the subgraph to delete")]
    pub name: String,
}
