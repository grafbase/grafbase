#[derive(Debug, clap::Args)]
pub struct IntrospectCommand {
    /// GraphQL URL to introspect
    url: String,
    /// Add a header to the introspection request
    #[clap(short = 'H', long, value_parser, num_args = 0..)]
    header: Vec<String>,
}

impl IntrospectCommand {
    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn headers(&self) -> impl Iterator<Item = (&str, &str)> {
        self.header.iter().filter_map(|header| super::split_header(header))
    }
}
