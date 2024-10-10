#[derive(Debug, clap::Args)]
pub struct IntrospectCommand {
    /// GraphQL URL to introspect
    pub(crate) url: String,
    /// Add a header to the introspection request
    #[clap(short = 'H', long, value_parser, num_args = 0..)]
    header: Vec<String>,
    /// Disable syntax highlighting of the introspected GraphQL
    #[clap(long)]
    pub(crate) no_color: bool,
}

impl IntrospectCommand {
    pub fn url(&self) -> &str {
        self.url.as_ref()
    }

    pub fn headers(&self) -> impl Iterator<Item = (&str, &str)> {
        self.header.iter().filter_map(|header| super::split_header(header))
    }
}
