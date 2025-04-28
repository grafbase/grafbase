use crate::{
    api::errors::{ApiError, CreateError, LoginApiError, PublishError},
    common::errors::CommonError,
    upgrade::UpgradeError,
};
use notify_debouncer_full::notify;
use std::{io, path::PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum CliError {
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
    /// returned if the account name argument provided to create is not an existing account
    #[error("could not find an account with the provided name")]
    NoAccountFound,
    #[error("error during graph introspection: {0}")]
    Introspection(String),
    #[error("could not read the trusted documents manifest: {0}")]
    TrustedDocumentsManifestReadError(#[source] io::Error),
    #[error(
        "could not parse the trusted documents manifest. Expecting a map from document id to GraphQL string or an Apollo Client manifest ({0})"
    )]
    TrustedDocumentsManifestParseError(#[source] serde_json::Error),
    #[error("could not read the GraphQL schema")]
    SchemaReadError(#[source] io::Error),
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
    #[error(transparent)]
    GenericError(#[from] anyhow::Error),
}

impl CliError {
    /// returns the appropriate hint for a [`CliError`]
    pub fn to_hint(&self) -> Option<String> {
        match self {
            Self::BackendApiError(
                ApiError::RequestError(_)
                | ApiError::CreateError(CreateError::Unknown(_))
                | ApiError::PublishError(PublishError::Unknown(_)),
            ) => Some("you may be using an older version of the Grafbase CLI, try updating".to_owned()),
            Self::BackendApiError(ApiError::NotLoggedIn) | Self::CommonError(CommonError::CorruptCredentialsFile) => {
                Some("try running 'grafbase login'".to_owned())
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

#[derive(Error, Debug)]
pub(crate) enum BackendError {
    // wraps a [`CommonError`]
    #[error(transparent)]
    CommonError(#[from] CommonError),
    #[error(transparent)]
    ApiError(#[from] ApiError),
    #[error(transparent)]
    Fatal(#[from] anyhow::Error),
    #[error("could not read the SDL from {0}\nCaused by: {1}")]
    ReadSdlFromFile(PathBuf, std::io::Error),
    #[error("could not introspect a subgraph URL: {0}")]
    IntrospectSubgraph(String),
    #[error("no url or schema_path were defined for an overridden subgraph: {0}")]
    NoDefinedRouteToSubgraphSdl(String),
    #[error("could not parse a subgraph:\n{0:#}")]
    ParseSubgraphSdl(#[from] cynic_parser::Error),
    #[error("could not start the federated gateway\nCaused by: {0}")]
    Serve(federated_server::Error),
    #[error("could not compose subgraphs\nCaused by: {0}")]
    Composition(String),
    #[error("could not fetch the specified branch")]
    FetchBranch,
    #[error("the specified branch does not exist")]
    BranchDoesntExist,
    #[error("could not set up a file watcher\nCaused by: {0}")]
    SetUpWatcher(notify::Error),
    #[error("could not determine the path of the home directory")]
    HomeDirectory,
    #[error("could not unpack CLI app\nCaused by: {0}")]
    UnpackCliAppArchive(std::io::Error),
    #[error("could not write the current version of the unpacked CLI app assets\nCaused by: {0}")]
    WriteAssetVersion(std::io::Error),
    #[error("could not read the current version of the unpacked CLI app assets\nCaused by: {0}")]
    ReadAssetVersion(std::io::Error),
    #[error("could not create ~/.grafbase\nCaused by: {0}")]
    CreateDotGrafbaseDirectory(std::io::Error),
    #[error("could not access ~/.grafbase\nCaused by: {0}")]
    AccessDotGrafbaseDirectory(std::io::Error),
    #[error("{0}")]
    Error(String),
}
