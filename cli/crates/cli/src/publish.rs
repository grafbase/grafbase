use crate::{cli_input::PublishCommand, errors::CliError, output::report};
use std::fs;

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
    let schema_path = schema_path.ok_or_else(|| CliError::MissingArgument("schema"))?;

    let schema = fs::read_to_string(schema_path).map_err(CliError::SchemaReadError)?;

    report::publishing();

    backend::api::publish::publish(
        project_ref.account(),
        project_ref.project(),
        project_ref.branch(),
        &subgraph_name,
        url.as_str(),
        &schema,
    )
    .await
    .map_err(CliError::BackendApiError)?;

    report::publish_command_success(&subgraph_name);

    Ok(())
}
