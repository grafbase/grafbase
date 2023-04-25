use crate::{errors::CliError, output::report};
use backend::project;
use common::environment::Environment;

pub fn init(name: Option<&str>, template: Option<&str>, no_home: bool) -> Result<(), CliError> {
    project::init(name, template).map_err(CliError::BackendError)?;
    Environment::try_init(no_home).map_err(CliError::CommonError)?;
    server::export_embedded_files(&Environment::get().user_dot_grafbase_path).map_err(CliError::ServerError)?;
    report::project_created(name);
    Ok(())
}
