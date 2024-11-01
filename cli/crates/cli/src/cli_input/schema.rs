use super::FullGraphRef;
use clap::Parser;

/// Fetch the GraphQL schema of a published subgraph by name, or the federated graph schema.
#[derive(Debug, Parser)]
pub struct SchemaCommand {
    #[arg(help = FullGraphRef::ARG_DESCRIPTION)]
    pub graph_ref: FullGraphRef,

    /// The name of the subgraph to fetch. If this is left out, the federated graph is fetched.
    #[arg(long("name"))]
    pub subgraph_name: Option<String>,
}
