use super::{graph_ref::FullOrPartialGraphRef, FullGraphRef};
use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
pub(crate) struct TrustCommand {
    #[arg(help = FullGraphRef::ARG_DESCRIPTION)]
    pub(crate) graph_ref: FullOrPartialGraphRef,
    /// The path to the manifest file
    #[clap(long, short = 'm')]
    pub(crate) manifest: PathBuf,
    /// The name of the client
    #[clap(long, short = 'c')]
    pub(crate) client_name: String,
}
