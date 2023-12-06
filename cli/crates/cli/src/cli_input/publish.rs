use super::ProjectRef;
use clap::{ArgGroup, Parser};
use url::Url;

/// Publish a subgraph
#[derive(Debug, Parser)]
#[clap(
    group(
        ArgGroup::new("dev-or-production")
            .required(true)
            .args(&["dev", "project_ref"])
    ),
    group(
        ArgGroup::new("dev-or-schema")
            .required(true)
            .args(&["dev", "schema_path"])
    )
)]
pub struct PublishCommand {
    #[arg(help = ProjectRef::ARG_DESCRIPTION)]
    pub(crate) project_ref: Option<ProjectRef>,

    /// Publish to a running development server
    #[arg(long)]
    pub(crate) dev: bool,

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

    /// The listening port of the federated dev
    #[arg(long, default_value_t = 4000)]
    pub(crate) dev_api_port: u16,

    /// Add a header to the introspection request
    #[clap(short = 'H', long, value_parser, num_args = 0..)]
    header: Vec<String>,
}

impl PublishCommand {
    pub(crate) fn headers(&self) -> impl Iterator<Item = (&str, &str)> {
        self.header.iter().filter_map(|header| super::split_header(header))
    }
}
