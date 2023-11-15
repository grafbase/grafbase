use ulid::Ulid;

#[derive(Debug, clap::Args)]
pub struct LinkCommand {
    /// The id of the linked project
    #[arg(short, long, value_name = "PROJECT_ID")]
    pub project: Option<Ulid>,
}
