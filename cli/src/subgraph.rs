use crate::{api, cli_input::FullGraphRef, errors::CliError, output::report};

#[tokio::main]
pub(super) async fn list(graph_ref: FullGraphRef) -> Result<(), CliError> {
    let (branch, subgraphs) = api::subgraph::list(graph_ref.account(), graph_ref.graph(), graph_ref.branch())
        .await
        .map_err(CliError::BackendApiError)?;

    report::subgraph_list_command_success(&branch, subgraphs.iter().map(|subgraph| subgraph.name.as_str()));

    Ok(())
}

#[tokio::main]
pub(super) async fn delete(graph_ref: FullGraphRef, subgraph_name: String) -> Result<(), CliError> {
    let branch = graph_ref.branch().ok_or(CliError::MissingArgument("branch"))?;

    api::subgraph::delete(graph_ref.account(), graph_ref.graph(), branch, &subgraph_name)
        .await
        .map_err(CliError::BackendApiError)?;

    report::subgraph_delete_success(&subgraph_name);

    Ok(())
}
