use crate::create::CreateArguments;

use super::ArgumentNames;

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

impl ArgumentNames for CreateCommand {
    fn argument_names(&self) -> Option<Vec<&'static str>> {
        let arguments = [(self.name.is_some(), vec!["name", "account", "self-hosted"])]
            .iter()
            .filter(|arguments| arguments.0)
            .flat_map(|arguments| arguments.1.clone())
            .collect::<Vec<_>>();
        if arguments.is_empty() {
            None
        } else {
            Some(arguments)
        }
    }
}
