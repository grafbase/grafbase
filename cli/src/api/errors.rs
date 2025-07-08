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

    /// returned if ~/.grafbase/credentials.json could not be read
    #[error("could not read '~/.grafbase/credentials.json'\nCaused by: {0}")]
    ReadCredentialsFile(io::Error),

    /// returned if ~/.grafbase could not be read
    #[error("could not read '~/.grafbase'\nCaused by: {0}")]
    ReadUserDotGrafbaseFolder(io::Error),

    /// returned if the provided access token is corrupt
    #[error("could not complete the action as your access token is corrupt")]
    CorruptAccessToken,

    /// returned if an operation failed due to a token being unauthorized or the user previously being deleted
    #[error("unauthorized or deleted user")]
    UnauthorizedOrDeletedUser,

    /// returned if a cynic request could not be completed
    #[error("could not complete a request: {0}")]
    RequestError(String),

    /// returned if a cynic request could not be completed (due to connection issues)
    #[error("could not complete a request")]
    ConnectionError,

    /// returned if ~/.grafbase could not be created
    #[error("could not create '~/.grafbase'\nCaused by: {0}")]
    CreateUserDotGrafbaseFolder(io::Error),

    /// returned if an available port could not be find
    #[error("could not find an available port")]
    FindAvailablePort,

    /// wraps a [`CreateError`]
    #[error(transparent)]
    CreateError(#[from] CreateError),

    /// wraps a [`PublishError`]
    #[error(transparent)]
    PublishError(#[from] PublishError),

    /// wraps a [`BranchError`]
    #[error(transparent)]
    BranchError(#[from] BranchError),

    /// returned if the graph does not exist
    #[error("could not find the graph")]
    GraphDoesNotExist,

    #[error("the graph is not self-hosted")]
    GraphNotSelfHosted,

    #[error("{0}")]
    SubgraphsError(String),
}

#[derive(Error, Debug)]
pub enum CreateError {
    /// returned if the given slug for a new graph is already in use
    #[error("could not create a new graph as the provided slug is already in use")]
    SlugAlreadyExists,

    /// returned if the given slug for a new graph is invalid
    #[error("could not create a new graph as the provided slug is invalid")]
    SlugInvalid,

    /// returned if the given slug for a new graph was too long
    #[error("could not create a new graph as the provided slug is longer than {max_length} characters")]
    SlugTooLong { max_length: i32 },

    /// returned if a given account ID does not exist
    #[error("could not create a new graph as the specified account ID does not exist")]
    AccountDoesNotExist,

    /// returned if the user has reached the current plan limit
    #[error("could not create a new graph as the current plan limit of {max} graphs has been reached")]
    CurrentPlanLimitReached { max: i32 },

    /// returned if the account selected for graph creation is disabled
    #[error("could not create a new graph as the selected account is disabled")]
    DisabledAccount,

    /// returned if an unknown error occurs
    #[error("could not create a new graph, encountered an unknown error\nCaused by: {0}")]
    Unknown(String),
}

#[derive(Error, Debug)]
pub enum PublishError {
    /// returned if provided branch does not exist
    #[error("provided branch does not exist in the graph")]
    BranchDoesNotExist,

    /// returned if an unknown error occurs
    #[error("could not publish, encountered an unknown error\nCaused by: {0}")]
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

#[derive(Error, Debug)]
pub enum BranchError {
    /// returned if the given branch does not exist
    #[error("branch {0} does not exist")]
    BranchDoesNotExist(String),
    #[error("branch {0} already exists")]
    BranchAlreadyExists(String),
    /// returned, if trying to delete the production branch
    #[error("branch `{0}` is the production branch of the graph, and cannot be deleted")]
    CannotDeleteProductionBranch(String),
    /// returned if an unknown error occurs
    #[error("could not delete branch, encountered an unknown error\nCaused by: {0}")]
    Unknown(String),
}
