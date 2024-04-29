use crate::{cli_input::PublishCommand, errors::CliError, output::report};
use std::{
    fs,
    io::{IsTerminal, Read},
};

#[tokio::main]
pub(crate) async fn publish(
    PublishCommand {
        subgraph_name,
        project_ref,
        url,
        schema_path,
        ..
    }: PublishCommand,
) -> Result<(), CliError> {
    let project_ref = project_ref.ok_or_else(|| CliError::MissingArgument("PROJECT_REF"))?;
    let schema = match schema_path {
        Some(path) => fs::read_to_string(path).map_err(CliError::SchemaReadError)?,
        None if std::io::stdin().is_terminal() => {
            return Err(CliError::MissingArgument("--schema or a schema piped through stdin"))
        }
        None => {
            let mut schema = String::new();

            std::io::stdin()
                .read_to_string(&mut schema)
                .map_err(CliError::SchemaReadError)?;

            schema
        }
    };

    report::publishing();

    let outcome = backend::api::publish::publish(
        project_ref.account(),
        project_ref.graph(),
        project_ref.branch(),
        &subgraph_name,
        url.as_str(),
        &schema,
    )
    .await
    .map_err(CliError::BackendApiError)?;

    match &outcome {
        backend::api::publish::PublishOutcome::Success { composition_errors } if composition_errors.is_empty() => {
            report::publish_command_success(&subgraph_name);
        }
        backend::api::publish::PublishOutcome::Success { composition_errors } => {
            report::publish_command_composition_failure(composition_errors);
        }
        backend::api::publish::PublishOutcome::ProjectDoesNotExist {
            account_name,
            project_name,
        } => report::publish_project_does_not_exist(account_name, project_name),
    };

    Ok(())
}
