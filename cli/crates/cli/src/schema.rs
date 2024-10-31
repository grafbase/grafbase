use crate::{cli_input::SchemaCommand, errors::CliError, output::report};

#[tokio::main]
pub(crate) async fn schema(cmd: SchemaCommand) -> Result<(), CliError> {
    let SchemaCommand {
        graph_ref,
        subgraph_name,
    } = cmd;
    let schema = backend::api::schema::schema(
        graph_ref.account(),
        graph_ref.graph(),
        graph_ref.branch(),
        subgraph_name.as_deref(),
    )
    .await
    .map_err(CliError::BackendApiError)?;

    report::schema_command_success(schema.as_deref());

    Ok(())
}
