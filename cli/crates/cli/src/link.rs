use crate::{errors::CliError, output::report, prompts::handle_inquire_error};
use backend::api::{
    link::{self, project_link_validations},
    types::{AccountWithProjects, Project},
};
use inquire::Select;
use std::fmt::Display;
use ulid::Ulid;

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
pub async fn link(project_id: Option<Ulid>) -> Result<(), CliError> {
    link_impl(project_id).await
}

pub async fn link_impl(project_id: Option<Ulid>) -> Result<(), CliError> {
    project_link_validations().await.map_err(CliError::BackendApiError)?;

    if let Some(project_id) = project_id {
        link::link_project(project_id.to_string())
            .await
            .map_err(CliError::BackendApiError)?;

        report::linked_non_interactive();

        return Ok(());
    }

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

    link::link_project(selected_project.id)
        .await
        .map_err(CliError::BackendApiError)?;

    report::linked(&selected_project.slug);

    Ok(())
}
