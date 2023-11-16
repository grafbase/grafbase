use super::ProjectRef;
use clap::Parser;

/// Fetch the GraphQL schema of a published subgraph by name, or the federated graph schema.
#[derive(Debug, Parser)]
pub struct SchemaCommand {
    #[arg(help = ProjectRef::ARG_DESCRIPTION)]
    pub project_ref: ProjectRef,

    /// The name of the subgraph to fetch. If this is left out, the federated graph is fetched.
    #[arg(long("name"))]
    pub subgraph_name: Option<String>,
}
