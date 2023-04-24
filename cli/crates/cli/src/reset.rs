use backend::project;

use crate::{errors::CliError, output::report};

pub fn reset() -> Result<(), CliError> {
    project::reset().map_err(CliError::BackendError)?;
    report::project_reset();
    Ok(())
}
