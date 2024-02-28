use cynic::http::CynicReqwestError;
use std::{io, path::PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    /// returned if the login server could not be started
    #[error("could not start the login server")]
    StartLoginServer,

    /// returned if the user is not logged in when attempting to use a command requiring auth
    #[error("could not proceed as you are not logged in")]
    NotLoggedIn,

    /// returned if ~/.grafbase/credentials.json could not be deleted
    #[error("could not delete '~/.grafbase/credentials.json'\nCaused by: {0}")]
    DeleteCredentialsFile(io::Error),

    /// returned if ~/.grafbase/project.json could not be deleted
    #[error("could not delete '~/.grafbase/project.json'\nCaused by: {0}")]
    DeleteProjectMetadataFile(io::Error),

    /// returned if ~/.grafbase/credentials.json could not be read
    #[error("could not read '~/.grafbase/credentials.json'\nCaused by: {0}")]
    ReadCredentialsFile(io::Error),

    /// returned if .grafbase/project.json could not be read
    #[error("could not read '.grafbase/project.json'\nCaused by: {0}")]
    ReadProjectMetadataFile(io::Error),

    /// returned if ~/.grafbase could not be read
    #[error("could not read '~/.grafbase'\nCaused by: {0}")]
    ReadUserDotGrafbaseFolder(io::Error),

    /// returned if .grafbase could not be read
    #[error("could not read '.grafbase'\nCaused by: {0}")]
    ReadProjectDotGrafbaseFolder(io::Error),

    /// returned if an operation failed due to the project not being linked
    #[error("could not complete the action as this project has not been linked")]
    UnlinkedProject,

    /// returned if the contents of the credential file are corrupt
    #[error("could not complete the action as your credential file is corrupt")]
    CorruptCredentialsFile,

    /// returned if the provided access token is corrupt
    #[error("could not complete the action as your access token is corrupt")]
    CorruptAccessToken,

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
    #[error("could not write the project metadata file\nCaused by: {0}")]
    WriteProjectMetadataFile(io::Error),

    /// returned if a cynic request could not be completed
    #[error("could not complete a request: {0}")]
    RequestError(String),

    /// returned if a mutation returns an entity of an unknown variant
    #[error("API returned unrecognised payload: {0}")]
    UnknownPayloadError(String),

    /// returned if a cynic request could not be completed (due to connection issues)
    #[error("could not complete a request")]
    ConnectionError,

    /// returned if a project being created has already been created
    #[error("could not proceed as this local project has already been linked to a remote project")]
    ProjectAlreadyLinked,

    /// returned if the path of ~/.grafbase could not be found
    #[error("could not find the current user home folder")]
    FindUserDotGrafbaseFolder,

    /// returned if ~/.grafbase could not be created
    #[error("could not create '~/.grafbase'\nCaused by: {0}")]
    CreateUserDotGrafbaseFolder(io::Error),

    /// returned if .grafbase could not be created
    #[error("could not create '.grafbase'\nCaused by: {0}")]
    CreateProjectDotGrafbaseFolder(io::Error),

    /// returned if an available port could not be find
    #[error("could not find an available port")]
    FindAvailablePort,

    /// returned if a the request to upload the archive fails
    #[error("could not complete the request to upload the deployment archive")]
    UploadError,

    /// returned if the upload archive metadata could not be read
    #[error("could not read the upload archive metadata\nCaused by: {0}")]
    ReadArchiveMetadata(io::Error),

    /// returned if the upload archive could not be read
    #[error("could not read the upload archive\nCaused by: {0}")]
    ReadArchive(io::Error),

    /// returned if a project file could not be read
    #[error("could not read a project file\nCaused by: {0}")]
    ReadProjectFile(ignore::Error),

    /// returned if a project file could not be opened
    #[error("could not open a project file\nCaused by: {0}")]
    OpenProjectFile(io::Error),

    /// returned if a file or directory could not be appended to the upload archive
    #[error("could not append a file or directory to the upload archive\nCaused by: {0}")]
    AppendToArchive(io::Error),

    /// returned if the upload archive could not be written
    #[error("could not write the upload archive\nCaused by: {0}")]
    WriteArchive(io::Error),

    /// returned if a temporary file for the upload archive could not be created
    #[error("could not create a temporary file\nCaused by: {0}")]
    CreateTempFile(io::Error),

    /// wraps a [`CreateError`]
    #[error(transparent)]
    CreateError(#[from] CreateError),

    /// wraps a [`DeployError`]
    #[error(transparent)]
    DeployError(#[from] DeployError),

    /// wraps a [`PublishError`]
    #[error(transparent)]
    PublishError(#[from] PublishError),

    /// returned if the project does not exist
    #[error("could not find the project")]
    ProjectDoesNotExist,

    #[error("{0}")]
    SubgraphsError(String),
}

#[derive(Error, Debug)]
pub enum CreateError {
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

    /// returned if invalid environment variables are used
    #[error("could not create a new project as invalid environment variables were supplied")]
    InvalidEnvironmentVariables,

    /// returned if the amount of enrionment variables supplied is over the allowed limit
    #[error("could not create a new project as the amount of environment variables exceeded the allowed limit")]
    EnvironmentVariableCountLimitExceeded,

    /// returned if the account selected for project creation is disaled
    #[error("could not create a new project as the selected account is disabled")]
    DisabledAccount,

    /// returned if an unknown error occurs
    #[error("could not create a new project, encountered an unknown error\nCaused by: {0}")]
    Unknown(String),
}

#[derive(Error, Debug)]
pub enum PublishError {
    /// returned if provided branch does not exist
    #[error("provided branch does not exist in the project")]
    BranchDoesNotExist,

    /// returned if an unknown error occurs
    #[error("could not publish, encountered an unknown error\nCaused by: {0}")]
    Unknown(String),
}

#[derive(Error, Debug)]
pub enum DeployError {
    /// returned if the linked project does not exist
    #[error("could not deploy as the linked project does not exist")]
    ProjectDoesNotExist,

    /// returned if the uploaded archive size is over the allowed limit
    #[error("could not deploy as the created archive size is above the allowed limit of {limit} bytes")]
    ArchiveFileSizeLimitExceeded { limit: i32 },

    /// returned if the daily deployment count is passed
    #[error("could not deploy as you have reached the allowed daily deployemnt amount of {limit}")]
    DailyDeploymentCountLimitExceeded { limit: i32 },

    /// returned if an unknown error occurs
    #[error("could not deploy, encountered an unknown error\nCaused by: {0}")]
    Unknown(String),
}

#[derive(Error, Debug)]
pub enum LoginApiError {
    #[error("could not write '{0}'")]
    WriteCredentialFile(PathBuf),
}

impl From<CynicReqwestError> for ApiError {
    fn from(error: CynicReqwestError) -> Self {
        match error {
            CynicReqwestError::ReqwestError(error) if error.is_connect() => ApiError::ConnectionError,
            CynicReqwestError::ReqwestError(_) | CynicReqwestError::ErrorResponse(_, _) => {
                ApiError::RequestError(error.to_string())
            }
        }
    }
}
