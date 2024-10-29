use clap::Parser;
use url::Url;

/// Log into Grafbase
#[derive(Debug, Parser)]
pub struct LoginCommand {
    /// The URL of the Grafbase dashboard, defaults to the hosted version
    #[arg(short, long)]
    pub(crate) url: Option<Url>,
}
