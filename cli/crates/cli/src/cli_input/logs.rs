const DEFAULT_LOGS_LIMIT: u16 = 100;

#[derive(Debug, clap::Args)]
pub struct LogsCommand {
    /// The reference to a project: either `{account_slug}/{project_slug}`, `{project_slug}` for the personal account, or a URL to a deployed gateway.
    /// Defaults to the linked project if there's one.
    #[arg(value_name = "PROJECT_BRANCH")]
    pub project_branch: Option<String>,
    /// How many last entries to retrive
    #[arg(short, long, default_value_t = DEFAULT_LOGS_LIMIT)]
    pub limit: u16,
    /// Whether to disable polling for new log entries
    #[arg(long)]
    pub no_follow: bool,
}
