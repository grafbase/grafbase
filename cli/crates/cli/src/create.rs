use crate::{errors::CliError, output::report};
use backend::api::{
    create,
    types::{Account, DatabaseRegion},
};
use common::environment::Environment;
use inquire::{validator::Validation, Confirm, InquireError, Select, Text};
use slugify::slugify;
use std::{fmt::Display, process};

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

/// # Errors
#[tokio::main]
pub async fn create() -> Result<(), CliError> {
    let environment = Environment::get();

    let (accounts, available_regions, closest_region) = create::get_viewer_data_for_creation()
        .await
        .map_err(CliError::BackendApiError)?;

    let options: Vec<AccountSelection> = accounts.into_iter().map(AccountSelection).collect();

    let dir_name = environment
        .project_path
        .file_name()
        .expect("must exist")
        .to_string_lossy();

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
        .with_default(false)
        .prompt()
        .map_err(handle_inquire_error)?;

    if confirm {
        let domains = create::create(&selected_account.id, &project_name, &[selected_region.0])
            .await
            .map_err(CliError::BackendApiError)?;

        report::created(&project_name, &domains);
    }

    Ok(())
}

fn handle_inquire_error(error: InquireError) -> CliError {
    match error {
        InquireError::NotTTY => CliError::PromptNotTTY,
        InquireError::IO(error) => CliError::PromptIoError(error),
        // exit normally without panicking on ESC or CTRL+C
        InquireError::OperationCanceled | InquireError::OperationInterrupted => process::exit(0),
        InquireError::InvalidConfiguration(_) | InquireError::Custom(_) => unreachable!(),
    }
}
