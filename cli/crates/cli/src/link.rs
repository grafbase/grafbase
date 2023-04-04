use crate::{errors::CliError, output::report, prompts::handle_inquire_error};
use backend::api::{
    link,
    types::{AccountWithProjects, Project},
};
use inquire::Select;
use std::fmt::Display;

#[derive(Debug)]
struct AccountSelection(AccountWithProjects);

impl Display for AccountSelection {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_fmt(format_args!("{} ({})", self.0.name, self.0.slug))
    }
}

struct ProjectSelection(Project);

impl Display for ProjectSelection {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_fmt(format_args!("{}", self.0.slug))
    }
}

#[tokio::main]
pub async fn link() -> Result<(), CliError> {
    let accounts = link::get_viewer_data_for_link()
        .await
        .map_err(CliError::BackendApiError)?;

    let options: Vec<AccountSelection> = accounts.into_iter().map(AccountSelection).collect();

    let AccountSelection(selected_account) =
        Select::new("Which account owns the project you'd like to link to?", options)
            .prompt()
            .map_err(handle_inquire_error)?;

    if selected_account.projects.is_empty() {
        return Err(CliError::AccountWithNoProjects);
    }

    let options: Vec<ProjectSelection> = selected_account.projects.into_iter().map(ProjectSelection).collect();

    let ProjectSelection(selected_project) = Select::new("Which project would you like to link to?", options)
        .prompt()
        .map_err(handle_inquire_error)?;

    link::link_project(selected_account.id, selected_project.id)
        .await
        .map_err(CliError::BackendApiError)?;

    report::linked(&selected_project.slug);

    Ok(())
}
