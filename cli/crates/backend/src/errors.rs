use cynic::http::CynicReqwestError;
pub use server::errors::ServerError;
use std::{io, path::PathBuf};
use thiserror::Error;

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
    #[error("could not clean the files extracted from the repository archive\ncaused by: {0}")]
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

    /// returned if the path of ~/.grafbase could not be found
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

    /// returned if the user is not logged in when attempting to log out
    #[error("could not log out as you are not logged in")]
    NotLoggedIn,

    /// returned if ~/.grafbase could not be created
    #[error("could not delete '~/.grafbase/credentials.json'\ncaused by: {0}")]
    DeleteCredentialsFile(io::Error),

    /// returned if ~/.grafbase/credentials.json could not be read
    #[error("could not read '~/.grafbase/credentials.json'\ncaused by: {0}")]
    ReadCredentialsFile(io::Error),

    /// returned if .grafbase/project.json could not be read
    #[error("could not read '.grafbase/project.json'\ncaused by: {0}")]
    ReadProjectMetadataFile(io::Error),

    /// returned if ~/.grafbase could not be read
    #[error("could not read '~/.grafbase'\ncaused by: {0}")]
    ReadUserDotGrafbaseFolder(io::Error),

    /// returned if an operation failed due to the user being logged out
    #[error("could not complete the action as you are logged out")]
    LoggedOut,

    /// returned if the contents of the credential file are corrupt
    #[error("could not complete the action as your credential file is corrupt")]
    CorruptCredentialsFile,

    /// returned if the contents of the project metadata file are corrupt
    #[error("could not complete the action as your project metadata file are corrupt")]
    CorruptProjectMetadataFile,

    /// returned if an operation failed due to a token being unauthorized or the user previously being deleted
    #[error("unauthorized or deleted user")]
    UnauthorizedOrDeletedUser,

    /// returned if a token does not have access to a user's personal account
    #[error("incorrectly scoped token")]
    IncorrectlyScopedToken,

    /// returned if a project schema could not be read
    #[error("could not read the project graphql schema")]
    ReadSchema,

    /// returned if the project metadata file could not be written
    #[error("could not write the project metadata file\ncaused by: {0}")]
    WriteProjectMetadataFile(io::Error),

    /// TODO hint regarding CLI version
    /// returned if a cynic request could not be completed
    #[error("could not complete a request")]
    RequestError,

    /// returned if a cynic request could not be completed (due to connection issues)
    #[error("could not complete a request")]
    ConnectionError,

    /// returned if a project being created has already been created
    #[error("could not create a new project as this local project has already been linked to a remote project")]
    ProjectAlreadyLinked,

    /// wraps a [`CreateApiError`]
    #[error("{0}")]
    CreateApiError(CreateApiError),
}

#[derive(Error, Debug)]
pub enum CreateApiError {
    /// returned if the given slug for a new project is already in use
    #[error("could not create a new project as the provided slug is already in use")]
    SlugAlreadyExists,

    /// returned if the given slug for a new project is invalid
    #[error("could not create a new project as the provided slug is invalid")]
    SlugInvalid,

    /// returned if the given slug for a new project was too long
    #[error("could not create a new project as the provided slug is longer than {max_length} characters")]
    SlugTooLong { max_length: i32 },

    /// returned if a given account ID does not exist
    #[error("could not create a new project as the specified account ID does not exist")]
    AccountDoesNotExist,

    /// returned if the user has reached the current plan limit
    #[error("could not create a new project as the current plan limit of {max} projects has been reached")]
    CurrentPlanLimitReached { max: i32 },

    /// returned if duplicate database regions were selected
    #[error("could not create a new project as duplicate database regions were selected")]
    DuplicateDatabaseRegions { duplicates: Vec<String> },

    /// returned if no database regions are selected
    #[error("could not create a new project as no database regions were selected")]
    EmptyDatabaseRegions,

    /// returned if invalid database regions are used
    #[error("could not create a new project as invalid regions were selected")]
    InvalidDatabaseRegions { invalid: Vec<String> },

    /// returned if an unknown error occurs
    #[error("could not create a new project, encountered an unknown error")]
    Unknown,
}

impl From<CreateApiError> for BackendError {
    fn from(error: CreateApiError) -> BackendError {
        BackendError::CreateApiError(error)
    }
}

#[derive(Error, Debug)]
pub enum LoginApiError {
    #[error("could not write '{0}'")]
    WriteCredentialFile(PathBuf),
}

impl From<CynicReqwestError> for BackendError {
    fn from(error: CynicReqwestError) -> Self {
        match error {
            CynicReqwestError::ReqwestError(error) if error.is_connect() => BackendError::ConnectionError,
            CynicReqwestError::ReqwestError(_) | CynicReqwestError::ErrorResponse(_, _) => BackendError::RequestError,
        }
    }
}
