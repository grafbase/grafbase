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

    /// returned if the current directory path could not be read
    #[error("could not read the current path")]
    ReadCurrentDirectory,

    /// returned if the grafbase directory could not be created
    #[error("could not create the 'grafbase' directory\ncaused by: {0}")]
    CreateGrafbaseDirectory(io::Error),

    /// returned if the project directory could not be created
    #[error("could not create the project directory\ncaused by: {0}")]
    CreateProjectDirectory(io::Error),

    /// returned if a schema.graphql file could not be created
    #[error("could not create a schema.graphql file\ncaused by: {0}")]
    WriteSchema(io::Error),

    /// returned if the dot grafbase directory could not be deleted
    #[error("could not delete the .grafbase directory\ncaused by: {0}")]
    DeleteDotGrafbaseDirectory(io::Error),

    /// returned if the grafbase directory for the project could not be deleted
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

    // since this is checked by looking for the extracted files on disk (extraction errors are checked beforehand),
    // may have unlikely false positives if the files were deleted or moved by an external process immediately after extraction.
    //
    // TODO: consider adding an indicator that a file was extracted rather than checking on disk
    // TODO: consider splitting this into internal and external template errors for clarity
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
    #[error("could not clean the files extracted from the repository archiveio::Error")]
    CleanExtractedFiles(io::Error),

    /// returned if the request to get the information for a repository could not be sent
    #[error("could not get the repository information for {0}")]
    StartGetRepositoryInformation(String),

    /// returned if the request to get the information for a repository returned a non 200-299 status
    #[error("could not get the repository information for {0}")]
    GetRepositoryInformation(String),

    /// returned if the request to get the information for a repository returned a response that could not be parsed
    #[error("could not read the repository information for {0}")]
    ReadRepositoryInformation(String),

    /// returned if the path of `~/.grafbase` could not be found
    #[error("could not find the current user home folder")]
    FindUserDotGrafbaseFolder,

    /// returned if ~/.grafbase could not be created
    #[error("could not create '~/.grafbase\ncaused by: {0}")]
    CreateUserDotGrafbaseFolder(io::Error),

    /// returned if an available port could not be find
    #[error("could not find an available port")]
    FindAvailablePort,

    /// returned if the login server could not be started
    #[error("could not start the login server")]
    StartLoginServer,
}

impl ToExitCode for BackendError {
    fn to_exit_code(&self) -> i32 {
        match &self {
            Self::AvailablePort
            | Self::PortInUse(_)
            | Self::StartDownloadRepoArchive(_, _)
            | Self::StartGetRepositoryInformation(_)
            | Self::ReadRepositoryInformation(_)
            | Self::DownloadRepoArchive(_)
            | Self::ReadArchiveEntries
            | Self::GetRepositoryInformation(_)
            | Self::StartLoginServer => exitcode::UNAVAILABLE,
            Self::AlreadyAProject(_) | Self::ProjectDirectoryExists(_) => exitcode::USAGE,
            Self::UnsupportedTemplateURL(_) | Self::MalformedTemplateURL(_) | Self::TemplateNotFound => {
                exitcode::DATAERR
            }
            Self::ReadCurrentDirectory
            | Self::MoveExtractedFiles(_)
            | Self::DeleteDotGrafbaseDirectory(_)
            | Self::DeleteGrafbaseDirectory(_)
            | Self::WriteSchema(_)
            | Self::CreateGrafbaseDirectory(_)
            | Self::ExtractArchiveEntry(_)
            | Self::CleanExtractedFiles(_)
            | Self::CreateProjectDirectory(_)
            | Self::FindUserDotGrafbaseFolder
            | Self::CreateUserDotGrafbaseFolder(_)
            | Self::FindAvailablePort => exitcode::IOERR,

            Self::ServerError(inner) => inner.to_exit_code(),
        }
    }
}
