use clap::Parser;
use std::path::PathBuf;
use url::Url;

/// Start a local development server
#[derive(Debug, Parser)]
pub struct McpCommand {
    /// The URL of the GraphQL service
    pub(crate) url: Url,
    /// Add a header to the introspection request
    #[clap(short('H'), long, value_parser)]
    header: Vec<String>,
    /// GraphQL schema to use instead of relying on introspection.
    #[clap(short('s'), long)]
    pub(crate) schema: Option<PathBuf>,
    /// Whether mutations should be included in the MCP server.
    #[clap(long)]
    pub(crate) include_mutations: bool,
    /// The port to listen on for requests
    #[arg(short('p'), long("port"))]
    pub(crate) port: Option<u16>,
}

impl McpCommand {
    pub fn headers(&self) -> impl Iterator<Item = (&str, &str)> {
        self.header.iter().filter_map(|header| super::split_header(header))
    }
}
