use backend::api::errors::{ApiError, CreateError, LoginApiError, PublishError};
use backend::errors::ServerError;
use common::errors::CommonError;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

use crate::upgrade::UpgradeError;

#[derive(Error, Debug)]
pub enum CliError {
    // TODO: this might be better as `expect`
    /// returned if the development server panics
    #[error("{0}")]
    ServerPanic(String),
    /// wraps a server error
    #[error(transparent)]
    ServerError(ServerError),
    /// wraps an error originating in the local-backend crate api module
    #[error(transparent)]
    BackendApiError(ApiError),
    /// wraps an error originating in the local-backend crate's login API
    #[error(transparent)]
    LoginApiError(LoginApiError),
    /// wraps an error originating in the common crate
    #[error(transparent)]
    CommonError(CommonError),
    // TODO: this might be better as `expect`
    /// returned if the login server panics
    #[error("{0}")]
    LoginPanic(String),
    /// The CLI argument was missing
    #[error("The following required argument was not provided: {_0}")]
    MissingArgument(&'static str),
    /// returned if an interactive prompt fails due to the input device not being a TTY
    #[error("could not show an interactive prompt due to the input device not being a TTY")]
    PromptNotTTY,
    /// returned if an IO error is encountered when trying to display an interactive prompt
    #[error("encountered an IO error while showing an interactive prompt\nCaused by: {0}")]
    PromptIoError(io::Error),
    /// returned if an account selected for linking a project has no projects
    #[error("the selected account has no projects")]
    AccountWithNoProjects,
    /// returned if the account name argument provided to create is not an existing account
    #[error("could not find an account with the provided name")]
    NoAccountFound,
    /// returned if the schema parser failed to compile a file
    #[error("{0}")]
    CompilationError(String),
    #[error("error during graph introspection: {0}")]
    Introspection(String),
    #[error("could not read the trusted documents manifest: {0}")]
    TrustedDocumentsManifestReadError(#[source] io::Error),
    #[error("could not parse the trusted documents manifest. Expecting a map from document id to GraphQL string or an Apollo Client manifest ({0})")]
    TrustedDocumentsManifestParseError(#[source] serde_json::Error),
    #[error("could not read the GraphQL schema")]
    SchemaReadError(#[source] io::Error),
    #[error("error in publish: {0}")]
    Publish(String),
    #[error(transparent)]
    UpgradeError(#[from] UpgradeError),
    /// returned if the CLI was installed via a package manager and not directly (when trying to upgrade)
    #[error("could not upgrade grafbase as it was installed using a package manager")]
    NotDirectInstall,
    #[error(transparent)]
    LintInvalidSchema(#[from] graphql_lint::LinterError),
    /// returned if a linted schema could not be read
    #[error("could not read '{0}'\nCaused by: {1}")]
    ReadLintSchema(PathBuf, io::Error),
    /// returned if a directory or file without an extension is passed to lint
    #[error("attempted to lint a directory or a file without an extension")]
    LintNoExtension,
    /// returned if an unsupported extension is passed to lint
    #[error("attempted to lint a file with an unsupported extension: '{0}'")]
    LintUnsupportedFileExtension(String),
}

#[cfg(target_family = "windows")]
const WINDOWS_DIR_NOT_EMPTY_CODE: i32 = 145;

impl CliError {
    /// returns the appropriate hint for a [`CliError`]
    pub fn to_hint(&self) -> Option<String> {
        match self {
            Self::CommonError(CommonError::FindGrafbaseDirectory) => {
                Some("try running the CLI in your Grafbase project or any nested directory".to_owned())
            }
            Self::ServerError(ServerError::AvailablePortServer) => {
                Some("try supplying a larger port range to search by supplying a lower --port number".to_owned())
            }
            Self::ServerError(ServerError::PortInUse(_)) => {
                Some("try using a different --port number or supplying the --search flag".to_owned())
            }
            Self::BackendApiError(
                ApiError::RequestError(_)
                | ApiError::CreateError(CreateError::Unknown(_))
                | ApiError::PublishError(PublishError::Unknown(_)),
            ) => Some("you may be using an older version of the Grafbase CLI, try updating".to_owned()),
            Self::BackendApiError(ApiError::NotLoggedIn | ApiError::CorruptCredentialsFile) => {
                Some("try running 'grafbase login'".to_owned())
            }
            Self::BackendApiError(ApiError::ProjectAlreadyLinked) => Some("try running 'grafbase deploy'".to_owned()),
            Self::BackendApiError(ApiError::CorruptProjectMetadataFile | ApiError::UnlinkedProject) => {
                Some("try running 'grafbase link'".to_owned())
            }
            Self::UpgradeError(UpgradeError::StartDownload | UpgradeError::StartGetLatestReleaseVersion) => {
                Some("this may be caused by connection issues".to_owned())
            }
            Self::NotDirectInstall => {
                Some("try upgrading via your original install method or installing grafbase directly".to_owned())
            }
            Self::LintUnsupportedFileExtension(_) | Self::LintNoExtension => Some(
                "try passing a file with a supported extension: '.gql', '.graphql', '.graphqls' or '.sdl'".to_owned(),
            ),
            _ => None,
        }
    }
}
