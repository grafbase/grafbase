use super::ProjectRef;
use clap::Parser;

/// List published subgraphs
#[derive(Debug, Parser)]
pub struct SubgraphsCommand {
    #[arg(help = ProjectRef::ARG_DESCRIPTION)]
    pub project_ref: ProjectRef,
}
