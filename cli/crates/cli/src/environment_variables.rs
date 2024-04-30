use backend::api::environment_variables;

use crate::{cli_input::GraphRefNoBranch, errors::CliError, output::report};

#[tokio::main]
pub async fn list(graph_ref: Option<GraphRefNoBranch>) -> Result<(), CliError> {
    let environment_variables = environment_variables::list(graph_ref.map(GraphRefNoBranch::into_parts))
        .await
        .map_err(CliError::BackendApiError)?;

    report::list_environment_variables(environment_variables);

    Ok(())
}

#[tokio::main]
pub async fn create<'a>(
    graph_ref: Option<GraphRefNoBranch>,
    name: &str,
    value: &str,
    branch_environment: impl IntoIterator<Item = &'a str>,
) -> Result<(), CliError> {
    environment_variables::create(
        graph_ref.map(GraphRefNoBranch::into_parts),
        name,
        value,
        branch_environment,
    )
    .await
    .map_err(CliError::BackendApiError)?;

    report::create_env_var_success();

    Ok(())
}

#[tokio::main]
pub async fn delete<'a>(
    graph_ref: Option<GraphRefNoBranch>,
    name: &str,
    branch_environment: impl IntoIterator<Item = &'a str>,
) -> Result<(), CliError> {
    environment_variables::delete(graph_ref.map(GraphRefNoBranch::into_parts), name, branch_environment)
        .await
        .map_err(CliError::BackendApiError)?;

    report::delete_env_var_success();

    Ok(())
}
