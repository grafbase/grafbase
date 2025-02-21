use crate::backend::api::{create, types::Account};
use crate::{errors::CliError, output::report, prompts::handle_inquire_error};
use inquire::{Confirm, Select, Text, validator::Validation};
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

    let (domains, graph_slug) = create::create(&account_id, arguments.name)
        .await
        .map_err(CliError::BackendApiError)?;

    report::create_success(arguments.name, &domains, arguments.account_slug, &graph_slug);

    Ok(())
}

async fn interactive() -> Result<(), CliError> {
    let accounts = create::get_viewer_data_for_creation()
        .await
        .map_err(CliError::BackendApiError)?;

    let options: Vec<AccountSelection> = accounts.into_iter().map(AccountSelection).collect();

    let graph_name = Text::new("What should your new graph be called?")
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

    let confirm = Confirm::new("Please confirm the above to create your new graph")
        .with_default(true)
        .prompt()
        .map_err(handle_inquire_error)?;

    if confirm {
        let (domains, graph_slug) = create::create(&selected_account.id, &graph_name)
            .await
            .map_err(CliError::BackendApiError)?;

        report::create_success(&graph_name, &domains, &selected_account.slug, &graph_slug);
    }

    Ok(())
}
