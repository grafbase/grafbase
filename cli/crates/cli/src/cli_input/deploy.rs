#[derive(Debug, clap::Args)]
pub struct DeployCommand {
    /// The branch of the graph. If omitted, defaults to the production branch.
    #[arg(short, long)]
    pub branch: Option<String>,
}
