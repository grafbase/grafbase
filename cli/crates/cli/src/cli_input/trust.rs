use super::ProjectRef;
use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
pub(crate) struct TrustCommand {
    #[arg(help = ProjectRef::ARG_DESCRIPTION)]
    pub(crate) project_ref: ProjectRef,
    /// The path to the manifest file
    #[clap(long, short = 'm')]
    pub(crate) manifest: PathBuf,
    /// The name of the client
    #[clap(long, short = 'c')]
    pub(crate) client_name: String,
}
