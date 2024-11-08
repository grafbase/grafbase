use super::FullGraphRef;

#[derive(Debug, clap::Args)]
pub struct CheckCommand {
    #[arg(help = FullGraphRef::ARG_DESCRIPTION)]
    pub graph_ref: FullGraphRef,
    /// The name of the subgraph to check
    #[arg(long("name"))]
    pub(crate) subgraph_name: String,

    /// The path to the GraphQL schema to check. If this is not provided, the schema will be read
    /// from stdin.
    #[arg(long)]
    pub schema: Option<String>,
}
