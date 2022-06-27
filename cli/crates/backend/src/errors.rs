use common::traits::ToExitCode;
use std::path::PathBuf;
use thiserror::Error;

pub use dev_server::errors::DevServerError;

#[derive(Error, Debug)]
pub enum BackendError {
    /// returned if no port is available.
    /// used specifically when searching for ports
    #[error("could not find an available port")]
    AvailablePort,
    /// returned if a given port is in use and the search option is not used
    #[error("port {0} is currently in use")]
    PortInUse(u16),
    /// wraps a dev server error
    #[error(transparent)]
    DevServerError(DevServerError),
    /// returned when trying to initialize a project that conflicts with an existing project
    #[error("{0} already exists")]
    AlreadyAProject(PathBuf),
    /// returned when trying to initialize a project that conflicts with an existing directory or file
    #[error("{0} already exists")]
    ProjectDirectoryExists(PathBuf),
    /// returned if the current directory path cannot be read
    #[error("could not read the current path")]
    ReadCurrentDirectory,
    /// returned if the grafbase directory cannot be created
    #[error("could not create a Grafbase directory")]
    CreateGrafbaseDirectory,
    /// returned if a schema.graphql file cannot be created
    #[error("could not create a schema.graphql file")]
    WriteSchema,
}

impl ToExitCode for BackendError {
    fn to_exit_code(&self) -> i32 {
        match &self {
            Self::AvailablePort | Self::PortInUse(_) => exitcode::UNAVAILABLE,
            Self::AlreadyAProject(_) | Self::ProjectDirectoryExists(_) => exitcode::USAGE,
            Self::ReadCurrentDirectory | Self::CreateGrafbaseDirectory | Self::WriteSchema => exitcode::DATAERR,
            Self::DevServerError(inner) => inner.to_exit_code(),
        }
    }
}
