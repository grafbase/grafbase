use crate::{errors::CliError, output::report};
use backend::project;
use common::environment::Environment;

pub fn reset() -> Result<(), CliError> {
    Environment::try_init().map_err(CliError::CommonError)?;
    project::reset().map_err(CliError::BackendError)?;
    report::project_reset();
    Ok(())
}
