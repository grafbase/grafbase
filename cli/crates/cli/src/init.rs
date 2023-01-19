use crate::{errors::CliError, output::report};
use backend::project;

pub fn init(name: Option<&str>, template: Option<&str>) -> Result<(), CliError> {
    project::init(name, template).map_err(CliError::BackendError)?;
    report::project_created(name);
    Ok(())
}
