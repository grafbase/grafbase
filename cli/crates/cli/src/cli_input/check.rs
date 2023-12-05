use super::ProjectRef;

#[derive(Debug, clap::Args)]
pub struct CheckCommand {
    #[arg(help = ProjectRef::ARG_DESCRIPTION)]
    pub project_ref: ProjectRef,
    /// The name of the subgraph to check. This argument is always required in a federated graph
    /// context, and it should not be used in a single graph context.
    #[arg(long)]
    pub subgraph: Option<String>,
    /// The path to the GraphQL schema to check.
    #[arg(long)]
    pub schema: Option<String>,
}
