use crate::create::CreateArguments;

#[derive(Debug, clap::Args)]
#[group(requires_all = ["name", "account"], multiple = true)]
pub struct CreateCommand {
    /// The name to use for the new graph
    #[arg(short, long)]
    pub name: Option<String>,
    /// The slug of the account in which the new graph should be created
    #[arg(short, long, value_name = "SLUG")]
    pub account: Option<String>,
}

impl CreateCommand {
    pub(crate) fn create_arguments(&self) -> Option<CreateArguments<'_>> {
        self.name
            .as_deref()
            .zip(self.account.as_deref())
            .map(|(name, account_slug)| CreateArguments { account_slug, name })
    }
}
