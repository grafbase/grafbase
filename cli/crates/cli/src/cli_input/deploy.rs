use clap::ArgGroup;

use super::ProjectRef;

#[derive(Debug, clap::Args)]
#[clap(group(ArgGroup::new("branch-or-graph-ref").required(false).args(["graph_ref", "branch"])))]
pub struct DeployCommand {
    #[arg(short, long, help = ProjectRef::ARG_DESCRIPTION)]
    pub graph_ref: Option<ProjectRef>,
    /// The branch to deploy into, if running from a linked project
    #[arg(short, long)]
    pub branch: Option<String>,
}
