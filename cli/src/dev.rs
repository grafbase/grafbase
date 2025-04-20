use crate::{backend, cli_input::DevCommand, errors::CliError};

pub(crate) fn dev(cmd: DevCommand) -> Result<(), CliError> {
    backend::dev::start(cmd).map_err(CliError::GenericError)
}
