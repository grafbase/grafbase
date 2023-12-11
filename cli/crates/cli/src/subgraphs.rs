use crate::{cli_input::SubgraphsCommand, errors::CliError, output::report};

#[tokio::main]
pub(super) async fn subgraphs(cmd: SubgraphsCommand) -> Result<(), CliError> {
    let project_ref = cmd.project_ref;
    let (branch, subgraphs) =
        backend::api::subgraphs::subgraphs(project_ref.account(), project_ref.project(), project_ref.branch())
            .await
            .map_err(CliError::BackendApiError)?;

    report::subgraphs_command_success(&branch, subgraphs.iter().map(|subgraph| subgraph.name.as_str()));

    Ok(())
}
