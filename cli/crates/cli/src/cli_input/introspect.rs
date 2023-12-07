#[derive(Debug, clap::Args)]
pub struct IntrospectCommand {
    /// GraphQL URL to introspect
    pub(crate) url: Option<String>,
    /// Add a header to the introspection request
    #[clap(short = 'H', long, value_parser, num_args = 0..)]
    header: Vec<String>,
    /// Pass this argument to introspect the local project. --url and --dev cannot be used together
    #[clap(long)]
    pub(crate) dev: bool,
}

impl IntrospectCommand {
    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }

    pub fn headers(&self) -> impl Iterator<Item = (&str, &str)> {
        self.header.iter().filter_map(|header| super::split_header(header))
    }
}
