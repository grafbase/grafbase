use backend::api::errors::{ApiError, CreateError, DeployError, LoginApiError, PublishError};
use backend::errors::{BackendError, ServerError};
use common::errors::CommonError;
use std::io::{self, ErrorKind};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    // TODO: this might be better as `expect`
    /// returned if the development server panics
    #[error("{0}")]
    ServerPanic(String),
    /// wraps a server error
    #[error(transparent)]
    ServerError(ServerError),
    /// wraps an error originating in the local-backend crate
    #[error(transparent)]
    BackendError(BackendError),
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
    /// returned if a command taking a project reference fails to find the project
    #[error("could not find the project referenced by: {0}")]
    ProjectNotFound(String),
    /// returned if the account name argument provided to create is not an existing account
    #[error("could not find an account with the provided name")]
    NoAccountFound,
    /// returned if the schema parser failed to compile a file
    #[error("{0}")]
    CompilationError(String),
    /// returned if `logs` is run without a project branch reference, and no project has been linked
    #[error("no project is linked to the workspace and `logs` has been invoked without any argument")]
    LogsNoLinkedProject,
    #[error("error during graph introspection: {0}")]
    Introspection(String),
    #[error("could not read the trusted documents manifest")]
    TrustedDocumentsManifestReadError(#[source] io::Error),
    #[error("could not read the GraphQL schema")]
    SchemaReadError(#[source] io::Error),
    #[error("error in publish: {0}")]
    Publish(String),
    /// returned if .grafbase/project.json could not be read
    #[error("could not read '.grafbase/project.json'\nCaused by: {0}")]
    ReadProjectMetadataFile(#[source] io::Error),
    /// the toml config load didn't succeed
    #[error("configuration parsing\n\n{0}")]
    ConfigurationError(String),
}

#[cfg(target_family = "windows")]
const WINDOWS_DIR_NOT_EMPTY_CODE: i32 = 145;

impl CliError {
    /// returns the appropriate hint for a [`CliError`]
    pub fn to_hint(&self) -> Option<String> {
        match self {
            Self::BackendError(BackendError::AlreadyAProject(_)) => Some("try running 'grafbase dev'".to_owned()),
            Self::BackendError(BackendError::DeleteDatabaseDirectory(error)) => {
                match error.kind() {
                    ErrorKind::NotFound => Some("this may be caused by the project previously being reset or by running 'grafbase reset' on a new project".to_owned()),
                    ErrorKind::PermissionDenied => Some("it appears that you do not have sufficient permissions to delete '.grafbase/database', try modifying its permissions".to_owned()),
                    // TODO: replace with ErrorKind::DirectoryNotEmpty once stable
                    #[cfg(target_family="windows")]
                    _ => error
                            .raw_os_error()
                            .filter(|raw| raw == &WINDOWS_DIR_NOT_EMPTY_CODE)
                            .map(|_| "this may be caused by '.grafbase/database' being in use by another instance of 'grafbase'".to_owned()),
                    #[cfg(target_family="unix")]
                    _ => None
                }
            }
            Self::BackendError(BackendError::DownloadRepoArchive(_)) => Some("this may be caused by an incorrect URL or trying to use a private repository as a template".to_owned()),
            Self::BackendError(BackendError::TemplateNotFound) => Some("this is likely to be caused by an incorrect template name or URL, or by an external template directory not containing a grafbase directory".to_owned()),
            Self::BackendError(BackendError::ProjectDirectoryExists(_)) => Some("try using a different name for your new project".to_owned()),
            Self::BackendError(BackendError::StartDownloadRepoArchive(_, _)) => Some("this may be caused by connection issues".to_owned()),
            Self::BackendError(BackendError::UnsupportedTemplateURL(_)) => Some("try using a GitHub URL of the following structure: 'https://github.com/org/repo'".to_owned()),
            Self::BackendError(BackendError::MalformedTemplateURL(_)) => Some("try including the URL scheme (e.g. 'https://') and verifying the URL contents".to_owned()),
            Self::CommonError(CommonError::FindGrafbaseDirectory) => Some("try running the CLI in your Grafbase project or any nested directory".to_owned()),
            Self::ServerError(ServerError::AvailablePortServer) => Some("try supplying a larger port range to search by supplying a lower --port number".to_owned()),
            Self::ServerError(ServerError::PortInUse(_)) => Some("try using a different --port number or supplying the --search flag".to_owned()),
            Self::BackendApiError(
                ApiError::RequestError(_)
                                | ApiError::CreateError(CreateError::Unknown(_))
                                | ApiError::DeployError(DeployError::Unknown(_))
                                | ApiError::PublishError(PublishError::Unknown(_))
            ) => Some("you may be using an older version of the Grafbase CLI, try updating".to_owned()),
            Self::BackendApiError(ApiError::NotLoggedIn | ApiError::CorruptCredentialsFile) => Some("try running 'grafbase login'".to_owned()),
            Self::BackendApiError(ApiError::ProjectAlreadyLinked) => Some("try running 'grafbase deploy'".to_owned()),
            Self::BackendApiError(ApiError::CorruptProjectMetadataFile | ApiError::UnlinkedProject) => Some("try running 'grafbase link'".to_owned()),
            _ => None,
        }
    }
}
