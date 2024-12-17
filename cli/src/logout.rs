use crate::{backend::api::logout, errors::CliError, output::report};

pub fn logout() -> Result<(), CliError> {
    logout::logout().map_err(CliError::BackendApiError)?;
    report::logout();
    Ok(())
}
