use clap::Parser;
use std::path::PathBuf;
use url::Url;

/// Start a GraphQL MCP server.
#[derive(Debug, Parser)]
pub struct McpCommand {
    /// The URL of the GraphQL service.
    pub(crate) url: Url,
    /// Add a header to the GraphQL requests.
    #[clap(short('H'), long, value_parser)]
    header: Vec<String>,
    /// GraphQL schema to use instead of relying on introspection.
    #[clap(short('s'), long)]
    pub(crate) schema: Option<PathBuf>,
    /// Grant this MCP server the ability to execute mutations.
    #[clap(long)]
    pub(crate) execute_mutations: bool,
    /// Port to listen on.
    #[arg(short('p'), long("port"))]
    pub(crate) port: Option<u16>,
}

impl McpCommand {
    pub fn headers(&self) -> impl Iterator<Item = (&str, &str)> {
        self.header.iter().filter_map(|header| super::split_header(header))
    }
}
