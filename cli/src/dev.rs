use crate::{backend, cli_input::DevCommand, errors::CliError};

pub(crate) fn dev(cmd: DevCommand) -> Result<(), CliError> {
    backend::dev::start(cmd.graph_ref, cmd.config_path, cmd.port).map_err(CliError::BackendError)
}
