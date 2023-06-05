use crate::{errors::CliError, output::report, prompts::handle_inquire_error};
use backend::api::{
    create,
    types::{Account, DatabaseRegion},
};
use common::environment::Project;
use inquire::{validator::Validation, Confirm, Select, Text};
use slugify::slugify;
use std::fmt::Display;

#[derive(Debug)]
struct AccountSelection(Account);

impl Display for AccountSelection {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_fmt(format_args!("{} ({})", self.0.name, self.0.slug))
    }
}

struct RegionSelection(DatabaseRegion);

impl Display for RegionSelection {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_fmt(format_args!("{} ({})", self.0.city, self.0))
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct CreateArguments<'a> {
    pub account_slug: &'a str,
    pub name: &'a str,
    pub regions: &'a [String],
}

#[tokio::main]
pub async fn create(arguments: &Option<CreateArguments<'_>>) -> Result<(), CliError> {
    match arguments {
        Some(arguments) => from_arguments(arguments).await,
        None => interactive().await,
    }
}

async fn from_arguments(arguments: &CreateArguments<'_>) -> Result<(), CliError> {
    report::create();

    // TODO do this with a separate mutation that accepts an account slug
    let (accounts, ..) = create::get_viewer_data_for_creation()
        .await
        .map_err(CliError::BackendApiError)?;

    let account_id = accounts
        .into_iter()
        .find(|account| account.slug == arguments.account_slug)
        .ok_or(CliError::NoAccountFound)?
        .id;

    let domains = create::create(&account_id, arguments.name, arguments.regions)
        .await
        .map_err(CliError::BackendApiError)?;

    report::create_success(arguments.name, &domains);

    Ok(())
}

async fn interactive() -> Result<(), CliError> {
    let project = Project::get();

    let (accounts, available_regions, closest_region) = create::get_viewer_data_for_creation()
        .await
        .map_err(CliError::BackendApiError)?;

    let options: Vec<AccountSelection> = accounts.into_iter().map(AccountSelection).collect();

    let dir_name = project.path.file_name().expect("must exist").to_string_lossy();

    let project_name = Text::new("What should your new project be called?")
        .with_default(&dir_name)
        .with_validator(|value: &str| {
            let slugified = slugify!(value, max_length = 48);
            if value == slugified {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid(
                    format!("Invalid project name, try '{slugified}'").into(),
                ))
            }
        })
        .prompt()
        .map_err(handle_inquire_error)?;

    let AccountSelection(selected_account) = Select::new("In which account should the project be created?", options)
        .prompt()
        .map_err(handle_inquire_error)?;

    let selected_region = Select::new(
        "In which region should your database be created?",
        available_regions.iter().cloned().map(RegionSelection).collect(),
    )
    .with_starting_cursor(
        available_regions
            .iter()
            .position(|region| region.name == closest_region.name)
            .unwrap_or_default(),
    )
    .prompt()
    .map_err(handle_inquire_error)?;

    let confirm = Confirm::new("Please confirm the above to create and deploy your new project")
        .with_default(true)
        .prompt()
        .map_err(handle_inquire_error)?;

    if confirm {
        let domains = create::create(&selected_account.id, &project_name, &[selected_region.0.name.clone()])
            .await
            .map_err(CliError::BackendApiError)?;

        report::create_success(&project_name, &domains);
    }

    Ok(())
}
