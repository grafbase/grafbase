use crate::{cli_input::DevCommand, errors::CliError};

pub fn dev(_cmd: DevCommand) -> Result<(), CliError> {
    Ok(())
}
