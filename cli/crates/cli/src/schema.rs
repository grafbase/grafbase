use crate::{cli_input::SchemaCommand, errors::CliError, output::report};

#[tokio::main]
pub(crate) async fn schema(cmd: SchemaCommand) -> Result<(), CliError> {
    let SchemaCommand {
        project_ref,
        subgraph_name,
    } = cmd;
    let schema = backend::api::schema::schema(
        project_ref.account(),
        project_ref.graph(),
        project_ref.branch(),
        subgraph_name.as_deref(),
    )
    .await
    .map_err(CliError::BackendApiError)?;

    report::schema_command_success(schema.as_deref());

    Ok(())
}
