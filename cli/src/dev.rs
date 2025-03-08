use crate::{backend, cli_input::DevCommand, errors::CliError};

pub(crate) fn dev(cmd: DevCommand) -> Result<(), CliError> {
    backend::dev::start(cmd.graph_ref, cmd.gateway_config, cmd.graph_overrides, cmd.port)
        .map_err(CliError::BackendError)
}
