use super::FullGraphRef;
use clap::{Args, Parser};
use url::Url;

/// Publish a subgraph
#[derive(Debug, Parser)]
pub(crate) struct PublishCommand {
    #[arg(help = FullGraphRef::ARG_DESCRIPTION)]
    pub(crate) graph_ref: FullGraphRef,

    /// The name of the subgraph
    #[arg(long("name"))]
    pub(crate) subgraph_name: String,

    /// The path to the GraphQL schema file to publish. If this argument is not provided, the
    /// schema will be read from stdin.
    #[arg(long("schema"))]
    pub(crate) schema_path: Option<String>,

    /// The message to annotate the publication with
    #[arg(long, short = 'm')]
    pub(crate) message: Option<String>,

    #[command(flatten)]
    pub(crate) source: SubgraphSource,
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub(crate) struct SubgraphSource {
    /// The URL to the GraphQL endpoint. Can be omitted if the subgraph is virtual and completely defined
    /// by an extension.
    #[arg(long)]
    pub(crate) url: Option<Url>,

    /// Subgraph does not exist, but is handled by an extension.
    #[arg(long)]
    pub(crate) r#virtual: bool,
}
