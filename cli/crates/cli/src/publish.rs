use crate::{cli_input::PublishCommand, errors::CliError, output::report};
use std::fs;

#[tokio::main]
pub(crate) async fn publish(
    PublishCommand {
        subgraph_name,
        project_ref,
        url,
        schema_path,
    }: PublishCommand,
) -> Result<(), CliError> {
    let schema = fs::read_to_string(schema_path).map_err(CliError::SchemaReadError)?;

    report::publishing();

    backend::api::publish::publish(
        project_ref.account(),
        project_ref.project(),
        project_ref.branch(),
        &subgraph_name,
        &url,
        &schema,
    )
    .await
    .map_err(CliError::BackendApiError)?;

    report::publish_command_success(&subgraph_name);

    Ok(())
}
