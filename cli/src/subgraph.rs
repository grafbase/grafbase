use crate::{
    api,
    cli_input::{SubgraphCommand, SubgraphSubCommand},
    errors::CliError,
    output::report,
};

#[tokio::main]
pub(super) async fn list(cmd: SubgraphCommand) -> Result<(), CliError> {
    let graph_ref = match cmd.command {
        SubgraphSubCommand::List { graph_ref } => graph_ref,
        _ => unreachable!("list called with non-list command"),
    };

    let (branch, subgraphs) = api::subgraph::list(graph_ref.account(), graph_ref.graph(), graph_ref.branch())
        .await
        .map_err(CliError::BackendApiError)?;

    report::subgraph_list_command_success(&branch, subgraphs.iter().map(|subgraph| subgraph.name.as_str()));

    Ok(())
}

#[tokio::main]
pub(super) async fn delete(cmd: SubgraphCommand, subgraph_name: String) -> Result<(), CliError> {
    let graph_ref = match cmd.command {
        SubgraphSubCommand::Delete { graph_ref, .. } => graph_ref,
        _ => unreachable!("delete called with non-delete command"),
    };

    let branch = graph_ref.branch().ok_or(CliError::MissingArgument("branch"))?;

    api::subgraph::delete(graph_ref.account(), graph_ref.graph(), branch, &subgraph_name)
        .await
        .map_err(CliError::BackendApiError)?;

    report::subgraph_delete_success(&subgraph_name);

    Ok(())
}
