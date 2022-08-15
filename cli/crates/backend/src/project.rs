use crate::consts::DEFAULT_SCHEMA;
use crate::errors::BackendError;
use common::consts::{GRAFBASE_DIRECTORY, GRAFBASE_SCHEMA};
use common::environment::Environment;
use std::env;
use std::fs;
use std::path::PathBuf;

/// initializes a new project in the current or a new directory
///
/// # Errors
///
/// - returns [`BackendError::ReadCurrentDirectory`] if the current directory cannot be read
///
/// - returns [`BackendError::ProjectDirectoryExists`] if a named is passed and a directory with the same name already exists in the current directory
///
/// - returns [`BackendError::AlreadyAProject`] if there's already a grafbase/schema.graphql in the target
///
/// - returns [`BackendError::CreateGrafbaseDirectory`] if the grafbase directory cannot be created
///
/// - returns [`BackendError::WriteSchema`] if the schema file cannot be written
pub fn init(name: Option<&str>) -> Result<(), BackendError> {
    let project_path = to_project_path(name)?;
    let grafbase_path = project_path.join(GRAFBASE_DIRECTORY);
    let schema_path = grafbase_path.join(GRAFBASE_SCHEMA);

    if schema_path.exists() {
        Err(BackendError::AlreadyAProject(schema_path))
    } else {
        fs::create_dir_all(&grafbase_path).map_err(|_| BackendError::CreateGrafbaseDirectory)?;
        fs::write(schema_path, DEFAULT_SCHEMA).map_err(|_| BackendError::WriteSchema)?;

        Ok(())
    }
}

/// resets the local data for the current project by removing the `.grafbase` directory
///
/// # Errors
///
/// - returns [`BackendError::ReadCurrentDirectory`] if the current directory cannot be read
///
/// - returns [`BackendError::DeleteDotGrafbaseDirectory`] if the `.grafbase` directory cannot be deleted
pub fn reset() -> Result<(), BackendError> {
    let environment = Environment::get();

    fs::remove_dir_all(environment.project_dot_grafbase_path.clone())
        .map_err(BackendError::DeleteDotGrafbaseDirectory)?;

    Ok(())
}

fn to_project_path(name: Option<&str>) -> Result<PathBuf, BackendError> {
    let current_dir = env::current_dir().map_err(|_| BackendError::ReadCurrentDirectory)?;

    match name {
        Some(name) => {
            let project_path = current_dir.join(name);
            if project_path.exists() {
                Err(BackendError::ProjectDirectoryExists(project_path))
            } else {
                Ok(project_path)
            }
        }
        None => Ok(current_dir),
    }
}
