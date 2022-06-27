use crate::{errors::CliError, output::report};
use backend::project;

pub fn init(name: Option<&str>) -> Result<(), CliError> {
    project::init(name).map_err(CliError::BackendError)?;
    report::project_created(name);
    Ok(())
}
