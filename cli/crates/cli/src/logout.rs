use crate::{errors::CliError, output::report};
use backend::api::logout;

pub fn logout() -> Result<(), CliError> {
    logout::logout().map_err(CliError::BackendApiError)?;
    report::logout();
    Ok(())
}
