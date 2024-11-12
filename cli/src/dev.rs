use crate::{cli_input::DevCommand, errors::CliError};

pub fn dev(cmd: DevCommand) -> Result<(), CliError> {
    // temporary, until we use `GraphRef`
    let full_graph_ref = cmd.graph_ref.map(|graph_ref| backend::dev::FullGraphRef {
        account: graph_ref.account().to_owned(),
        graph: graph_ref.graph().to_owned(),
        branch: graph_ref.branch().map(|branch| branch.to_owned()),
    });
    backend::dev::start(full_graph_ref, cmd.gateway_config, cmd.graph_overrides, cmd.port)
        .map_err(CliError::BackendError)
}
