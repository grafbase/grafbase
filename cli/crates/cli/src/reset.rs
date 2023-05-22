use crate::{errors::CliError, output::report};
use backend::project;

pub fn reset() -> Result<(), CliError> {
    project::reset().map_err(CliError::BackendError)?;
    report::project_reset();
    Ok(())
}
