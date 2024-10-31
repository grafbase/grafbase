use super::FullGraphRef;
use clap::Parser;
use url::Url;

/// Publish a subgraph
#[derive(Debug, Parser)]
pub struct PublishCommand {
    #[arg(help = FullGraphRef::ARG_DESCRIPTION)]
    pub(crate) graph_ref: FullGraphRef,

    /// The name of the subgraph
    #[arg(long("name"))]
    pub(crate) subgraph_name: String,

    /// The path to the GraphQL schema file to publish. If this argument is not provided, the
    /// schema will be read from stdin.
    #[arg(long("schema"))]
    pub(crate) schema_path: Option<String>,

    /// The URL to the GraphQL endpoint
    #[arg(long)]
    pub(crate) url: Url,

    /// The message to annotate the publication with
    #[arg(long, short = 'm')]
    pub(crate) message: Option<String>,

    /// Add a header to the introspection request
    #[clap(short = 'H', long, value_parser, num_args = 0..)]
    header: Vec<String>,
}
