use crate::{deploy, errors::CliError, output::report, prompts::handle_inquire_error};
use backend::api::{create, types::Account};
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

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct CreateArguments<'a> {
    pub account_slug: &'a str,
    pub name: &'a str,
    pub env_vars: Vec<(&'a str, &'a str)>,
}

#[tokio::main]
pub async fn create(arguments: &Option<CreateArguments<'_>>) -> Result<(), CliError> {
    create_impl(arguments).await
}

pub async fn create_impl(arguments: &Option<CreateArguments<'_>>) -> Result<(), CliError> {
    match arguments {
        Some(arguments) => from_arguments(arguments).await,
        None => interactive().await,
    }
}

async fn from_arguments(arguments: &CreateArguments<'_>) -> Result<(), CliError> {
    report::create();

    // TODO do this with a separate mutation that accepts an account slug
    let accounts = create::get_viewer_data_for_creation()
        .await
        .map_err(CliError::BackendApiError)?;

    let account_id = accounts
        .into_iter()
        .find(|account| account.slug == arguments.account_slug)
        .ok_or(CliError::NoAccountFound)?
        .id;

    let (domains, deployment_id) = create::create(&account_id, arguments.name, arguments.env_vars.iter().copied())
        .await
        .map_err(CliError::BackendApiError)?;

    deploy::report_progress(deployment_id.into_inner()).await?;
    report::create_success(arguments.name, &domains);

    Ok(())
}

async fn interactive() -> Result<(), CliError> {
    let project = Project::get();

    let accounts = create::get_viewer_data_for_creation()
        .await
        .map_err(CliError::BackendApiError)?;

    let options: Vec<AccountSelection> = accounts.into_iter().map(AccountSelection).collect();

    let dir_name = project.path.file_name().expect("must exist").to_string_lossy();

    let project_name = Text::new("What should your new graph be called?")
        .with_default(&dir_name)
        .with_validator(|value: &str| {
            let slugified = slugify!(value, max_length = 48);
            if value == slugified {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid(
                    format!("Invalid graph name, try '{slugified}'").into(),
                ))
            }
        })
        .prompt()
        .map_err(handle_inquire_error)?;

    let AccountSelection(selected_account) = Select::new("In which account should the graph be created?", options)
        .prompt()
        .map_err(handle_inquire_error)?;

    let confirm_env_vars = Confirm::new("Would you like to add environment variables to the graph?")
        .with_default(false)
        .prompt()
        .map_err(handle_inquire_error)?;

    let mut env_vars = Vec::new();

    if confirm_env_vars {
        loop {
            let key = Text::new("key (press enter to exit):")
                .with_default("")
                .prompt()
                .map_err(handle_inquire_error)?;

            if key.is_empty() {
                break;
            }

            let value = Text::new("value:")
                .with_default("")
                .prompt()
                .map_err(handle_inquire_error)?;

            env_vars.push((key, value));
        }
    }

    let confirm = Confirm::new("Please confirm the above to create and deploy your new graph")
        .with_default(true)
        .prompt()
        .map_err(handle_inquire_error)?;

    if confirm {
        let (domains, deployment_id) = create::create(
            &selected_account.id,
            &project_name,
            env_vars.iter().map(|(k, v)| (k.as_str(), v.as_str())),
        )
        .await
        .map_err(CliError::BackendApiError)?;

        deploy::report_progress(deployment_id.into_inner()).await?;
        report::create_success(&project_name, &domains);
    }

    Ok(())
}
