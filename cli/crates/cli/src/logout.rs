use crate::{errors::CliError, output::report};
use backend::logout;

pub fn logout() -> Result<(), CliError> {
    logout::logout().map_err(CliError::BackendError)?;
    report::logout();
    Ok(())
}
