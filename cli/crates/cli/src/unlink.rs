use crate::{errors::CliError, output::report};
use backend::api::unlink;

pub fn unlink() -> Result<(), CliError> {
    unlink::unlink().map_err(CliError::BackendApiError)?;

    report::unlinked();
    Ok(())
}
