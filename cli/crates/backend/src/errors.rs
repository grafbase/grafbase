use common::traits::ToExitCode;
use std::{io, path::PathBuf};
use thiserror::Error;

pub use server::errors::ServerError;

#[derive(Error, Debug)]
pub enum BackendError {
    /// returned if no port is available.
    /// used specifically when searching for ports
    #[error("could not find an available port")]
    AvailablePort,

    /// returned if a given port is in use and the search option is not used
    #[error("port {0} is currently in use")]
    PortInUse(u16),

    /// wraps a server error
    #[error(transparent)]
    ServerError(ServerError),

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
    #[error("could not create a Grafbase directory\ncaused by: {0}")]
    CreateGrafbaseDirectory(io::Error),

    /// returned if a schema.graphql file cannot be created
    #[error("could not create a schema.graphql file\ncaused by: {0}")]
    WriteSchema(io::Error),

    /// returned if the dot grafbase directory cannot be deleted
    #[error("could not delete the .grafbase directory\ncaused by: {0}")]
    DeleteDotGrafbaseDirectory(io::Error),

    /// returned if the grafbase directory for the project cannot be deleted
    #[error("could not delete the grafbase directory\ncaused by: {0}")]
    DeleteGrafbaseDirectory(io::Error),

    /// returned if a template URL is not supported
    #[error("'{0}' is not a supported template URL")]
    UnsupportedTemplateURL(String),

    /// returned if a template URL could not be parsed
    #[error("'{0}' is not a valid URL")]
    MalformedTemplateURL(String),

    /// returned if a repo tar could not be downloaded (on a non 200-299 status)
    #[error("could not download the archive for '{0}'\ncaused by: {1}")]
    StartDownloadRepoArchive(String, reqwest_middleware::Error),

    /// returned if a repo tar could not be downloaded
    #[error("could not download the archive for '{0}'")]
    DownloadRepoArchive(String),

    /// returned if a repo tar could not be stored
    #[error("could not store the archive for '{0}'\ncaused by: {1}")]
    StoreRepoArchive(String, std::io::Error),

    // since this is checked by looking for the extracted files on disk (extraction errors are checked beforehand),
    // may have unlikely false positives if the files were deleted or moved by an external process immediately after extraction.
    //
    // TODO: consider adding an indicator that a file was extracted rather than checking on disk
    // and change this error to something indicating that the extracted files were not found
    /// returned if no files matching the template path were extracted (excluding extraction errors)
    #[error("could not find the provided template within the template repository")]
    TemplateNotFound,

    /// returned if the extracted files from the template repository could not be moved
    #[error("could not move the extracted files from the template repository\ncaused by: {0}")]
    MoveExtractedFiles(io::Error),

    /// returned if the entries of the template repository archive could not be read
    #[error("could not read the entries of the template repository archive")]
    ReadArchiveEntries,

    /// returned if one of the entries of the template repository archive could not be extracted
    #[error("could not extract an entry from the template repository archive\ncaused by: {0}")]
    ExtractArchiveEntry(io::Error),

    /// returned if the files extracted from the template repository archive could not be cleaned
    #[error("could not clean the files extracted from the repository archive\ncaused by: {0}")]
    CleanExtractedFiles(io::Error),
}

impl ToExitCode for BackendError {
    fn to_exit_code(&self) -> i32 {
        match &self {
            Self::AvailablePort | Self::PortInUse(_) => exitcode::UNAVAILABLE,
            Self::AlreadyAProject(_) | Self::ProjectDirectoryExists(_) => exitcode::USAGE,
            Self::ReadCurrentDirectory
            | Self::CreateGrafbaseDirectory(_)
            | Self::WriteSchema(_)
            | Self::DeleteDotGrafbaseDirectory(_)
            | Self::DeleteGrafbaseDirectory(_)
            | Self::UnsupportedTemplateURL(_)
            | Self::MalformedTemplateURL(_)
            | Self::StartDownloadRepoArchive(_, _)
            | Self::DownloadRepoArchive(_)
            | Self::StoreRepoArchive(_, _)
            | Self::TemplateNotFound
            | Self::MoveExtractedFiles(_)
            | Self::ReadArchiveEntries
            | Self::ExtractArchiveEntry(_)
            | Self::CleanExtractedFiles(_) => exitcode::DATAERR,
            Self::ServerError(inner) => inner.to_exit_code(),
        }
    }
}
