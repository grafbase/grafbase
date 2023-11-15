use crate::create::CreateArguments;

use super::ArgumentNames;

#[derive(Debug, clap::Args)]
#[group(requires_all = ["name", "account", "regions"], multiple = true)]
pub struct CreateCommand {
    /// The name to use for the new project
    #[arg(short, long)]
    pub name: Option<String>,
    /// The slug of the account in which the new project should be created
    #[arg(short, long, value_name = "SLUG")]
    pub account: Option<String>,
    /// The regions in which the database for the new project should be created
    #[arg(short, long, value_name = "REGION")]
    pub regions: Option<Vec<String>>,
}

impl CreateCommand {
    pub fn create_arguments(&self) -> Option<CreateArguments<'_>> {
        self.name
            .as_deref()
            .zip(self.account.as_deref())
            .zip(self.regions.as_deref())
            .map(|((name, account_slug), regions)| CreateArguments {
                account_slug,
                name,
                regions,
            })
    }
}

impl ArgumentNames for CreateCommand {
    fn argument_names(&self) -> Option<Vec<&'static str>> {
        let arguments = [(self.name.is_some(), vec!["name", "account", "regions"])]
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
