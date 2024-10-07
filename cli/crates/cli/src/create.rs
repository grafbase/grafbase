use crate::{errors::CliError, output::report, prompts::handle_inquire_error};
use backend::api::{create, types::Account};
use common::environment::Project;
use inquire::{validator::Validation, Confirm, Select, Text};
use slugify::slugify;
use std::{fmt::Display, str::FromStr};

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
    pub(crate) graph_mode: GraphMode,
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

    let (domains, project_slug) = create::create(
        &account_id,
        arguments.name,
        arguments.graph_mode.into(),
        arguments.env_vars.iter().copied(),
    )
    .await
    .map_err(CliError::BackendApiError)?;

    report::create_success(arguments.name, &domains, arguments.account_slug, &project_slug);

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

    let graph_mode = Select::new(
        "Should the graph be managed or self-hosted?",
        vec![GraphMode::Managed, GraphMode::SelfHosted],
    )
    .with_starting_cursor(0)
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

    let maybe_and_deploy = match graph_mode {
        GraphMode::Managed => "and deploy ",
        GraphMode::SelfHosted => "",
    };

    let confirm = Confirm::new(&format!(
        "Please confirm the above to create {maybe_and_deploy}your new graph"
    ))
    .with_default(true)
    .prompt()
    .map_err(handle_inquire_error)?;

    if confirm {
        let (domains, project_slug) = create::create(
            &selected_account.id,
            &project_name,
            graph_mode.into(),
            env_vars.iter().map(|(k, v)| (k.as_str(), v.as_str())),
        )
        .await
        .map_err(CliError::BackendApiError)?;

        report::create_success(&project_name, &domains, &selected_account.slug, &project_slug);
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) enum GraphMode {
    #[default]
    Managed,
    SelfHosted,
}

impl std::fmt::Display for GraphMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rendered = match self {
            GraphMode::Managed => "Managed",
            GraphMode::SelfHosted => "Self-hosted",
        };

        f.write_str(rendered)
    }
}

impl From<GraphMode> for backend::api::create::GraphMode {
    fn from(value: GraphMode) -> Self {
        match value {
            GraphMode::Managed => backend::api::create::GraphMode::Managed,
            GraphMode::SelfHosted => backend::api::create::GraphMode::SelfHosted,
        }
    }
}

impl FromStr for GraphMode {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "self-hosted" => Ok(Self::SelfHosted),
            "managed" => Ok(Self::Managed),
            _ => Err("mode must be one of 'self-hosted' or 'managed'"),
        }
    }
}
