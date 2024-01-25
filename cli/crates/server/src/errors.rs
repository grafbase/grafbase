use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Json;
use common::types::UdfKind;
use notify::Error as NotifyError;
use serde_json::json;
use std::io::Error as IoError;
use std::path::PathBuf;
use thiserror::Error;
use tokio::task::JoinError;

use crate::config::ConfigError;
use crate::node::NodeError;
use crate::udf_builder::JavaScriptPackageManager;

#[derive(Error, Debug)]
pub enum JavascriptPackageManagerComamndError {
    #[error("working directory '{0}' not found")]
    WorkingDirectoryNotFound(PathBuf),
    #[error("working directory '{0}' cannot be read.\nCaused by: {1}")]
    WorkingDirectoryCannotBeRead(PathBuf, std::io::Error),
    /// returned if npm/pnpm/yarn cannot be found
    #[error("could not find {0}: {1}")]
    NotFound(JavaScriptPackageManager, which::Error),

    /// returned if any of the npm/pnpm/yarn commands exits unsuccessfully
    #[error("{0} encountered an error: {1}")]
    CommandError(JavaScriptPackageManager, IoError),

    /// returned if any of the npm/pnpm/yarn commands exits unsuccessfully
    #[error("{0} failed with output:\n{1}")]
    OutputError(JavaScriptPackageManager, String),
}

#[derive(Error, Debug)]
pub enum ServerError {
    /// returned if the directory cannot be read
    #[error("could not create path '{0}' for the embedded server files")]
    CreateDir(PathBuf),

    /// returned if any of the embedded worker files cannot be written to disk
    #[error("could not write an embedded server file: {0}")]
    WriteFile(String),

    /// returned if the version of the existing worker files cannot be read
    #[error("could not read the previously extracted embedded file versions")]
    ReadVersion,

    /// returned if the sqlite bridge cannot be started
    #[error("the bridge api could not be started: {0}")]
    StartBridgeApi(std::io::Error),

    /// returned if the error server cannot be started
    #[error("the error server could not be started: {0}")]
    StartErrorServer(std::io::Error),

    /// returned if the gateway server cannot be started
    #[error("the gateway server could not be started: {0}")]
    StartGatewayServer(std::io::Error),

    /// returned if the miniflare command returns an error
    #[error("miniflare encountered an error: {0}")]
    MiniflareCommandError(IoError),

    /// returned if the miniflare command exits unsuccessfully
    #[error("miniflare encountered an error\ncause:\n{0}")]
    MiniflareError(String),

    /// returned if the schema parser command returns an error
    #[error(transparent)]
    SchemaParserError(IoError),

    /// returned if reading the parser result fails
    #[error(transparent)]
    SchemaParserResultRead(IoError),

    /// returned if the schema parser result is invalid JSON
    #[error("schema parser result is malformed JSON:\n{0}")]
    SchemaParserResultJson(serde_json::Error),

    /// returned if writing the schema registry fails
    #[error(transparent)]
    SchemaRegistryWrite(IoError),

    /// returned if `tempfile::NamedTempFile::new()` fails.
    #[error("could not create a temporary file: {0}")]
    CreateTemporaryFile(IoError),

    /// returned if a write to a temporary file fails.
    #[error("could not write to a temporary file '{0}': {1}")]
    CreateNotWriteToTemporaryFile(PathBuf, IoError),

    /// returned if a read operation from a file fails
    #[error("could not read the file {0}: {1}")]
    ReadFile(PathBuf, IoError),

    /// returned if the schema parser command exits unsuccessfully
    #[error("{0}")]
    ParseSchema(String),

    /// returned if the typescript config parser command exits unsuccessfully
    #[error("could not load grafbase/grafbase.config.ts\nCaused by: {0}")]
    LoadTsConfig(String),

    #[error("could not find a resolver referenced in the schema under the path {0}.{{js,ts}}")]
    ResolverDoesNotExist(PathBuf),

    /// returned if any of the package manager commands ran during resolver build exits unsuccessfully
    #[error("command error: {0}")]
    WranglerInstallPackageManagerCommandError(#[from] JavascriptPackageManagerComamndError),

    /// returned if any of the npm commands ran during resolver build exits unsuccessfully
    #[error("resolver {0} failed to build:\n{1}")]
    ResolverBuild(String, String),

    /// returned if the user project path is not valid utf-8
    #[error("non utf-8 path used for project")]
    ProjectPath,

    /// returned if the user cache path is not valid utf-8
    #[error("$HOME/.grafbase is a non utf8 path")]
    CachePath,

    /// returned if the `.grafbase` directory cannot be created
    #[error("could not create a project cache directory")]
    CreateCacheDir,

    /// returned if the `.grafbase/database` directory cannot be created
    #[error("could not create a project database directory\nCaused by: {0}")]
    CreateDatabaseDir(IoError),

    /// returned if the `.grafbase/database` directory cannot be read
    #[error("could not read the project database directory\nCaused by: {0}")]
    ReadDatabaseDir(IoError),

    /// returned if an available port cannot be found for the bridge server or playground
    #[error("could not find an available port for an internal server")]
    AvailablePort,

    /// returned if no port is available.
    /// used specifically when searching for ports
    #[error("could not find an available port")]
    AvailablePortMiniflare,

    /// returned if a given port is in use and the search option is not used
    #[error("port {0} is currently in use")]
    PortInUse(u16),

    /// returned if a spawned task panics
    #[error(transparent)]
    SpawnedTaskPanic(#[from] JoinError),

    /// returned if node is not in the user $PATH
    #[error("Node.js does not seem to be installed")]
    NodeInPath,

    /// returned if the installed version of node is unsupported
    #[error("Node.js version {0} is unsupported")]
    OutdatedNode(String, String),

    /// returned if the installed version of node could not be retreived
    #[error("Could not retrive the installed version of Node.js")]
    CheckNodeVersion,

    /// returned if a file watcher could not be initialized or was stopped due to an error
    #[error("A file watcher encountered an error\nCaused by: {0}")]
    FileWatcher(#[from] NotifyError),

    /// returned if the proxy server could not be started
    #[error("could not start the proxy server\nCaused by:{0}")]
    StartProxyServer(std::io::Error),

    #[error("Could not create a lock for the wrangler installation: {0}")]
    Lock(fslock::Error),

    #[error("Could not release the lock for the wrangler installation: {0}")]
    Unlock(fslock::Error),

    #[error(transparent)]
    UdfBuildError(#[from] UdfBuildError),

    #[error("Error in gateway initialization: {0}")]
    GatewayError(String),

    #[error("Failed loading the federated graph from the SDL: {0}")]
    InvalidFederatedGraphSdl(String),
}

#[derive(Debug, Error)]
pub enum UdfBuildError {
    /// returned if `tempfile::NamedTempFile::new()` fails.
    #[error("could not create a temporary file for the parser result: {0}")]
    CreateTemporaryFile(IoError),

    /// path is invalid.
    #[error("path is invalid: {0}")]
    PathError(String),

    /// returned if a write to a UDF artifact file fails
    #[error("could not create a file {0} during a {1} build: {2}")]
    CreateUdfArtifactFile(PathBuf, UdfKind, IoError),

    /// returned if the directory cannot be created
    #[error("could not create path '{0}' for {1} build artifacts")]
    CreateDir(PathBuf, UdfKind),

    /// returned if a read operation from a file fails
    #[error("could not read the file {0}: {1}")]
    ReadFile(PathBuf, IoError),

    /// returned if the schema parser command exits unsuccessfully
    #[error("could not extract the {0} wrapper worker contents")]
    ExtractUdfWrapperWorkerContents(UdfKind, IoError),

    /// returned if a write to a temporary file fails.
    #[error("could not write to a temporary file '{0}': {1}")]
    CreateNotWriteToTemporaryFile(PathBuf, IoError),

    /// returned if creating symlink to a file fails.
    #[error("could not link to file: {0}")]
    SymlinkFailure(IoError),

    /// returned if kv data path is invalid. E.g: has non unicode characters
    #[error("invalid KV data path: {0}")]
    InvalidKvDataPath(String),

    #[error("could not find a {0} referenced in the schema under the path {1}.{{js,ts}}")]
    UdfDoesNotExist(UdfKind, PathBuf),

    /// returned if any of the package manager commands ran during resolver build exits unsuccessfully
    #[error("command error: {0}")]
    WranglerInstallPackageManagerCommandError(#[from] JavascriptPackageManagerComamndError),

    /// returned if the wrangler build step failed
    #[error("\n{output}")]
    WranglerBuildFailed { output: String },

    // returned if miniflare for a given UDF fails to spawn
    #[error("unknown spawn error")]
    MiniflareSpawnFailed,

    // returned if miniflare for a given UDF fails to spawn, with more details
    #[error("\n{output}")]
    MiniflareSpawnFailedWithOutput { output: String },

    /// returned if a spawned task panics
    #[error(transparent)]
    SpawnedTaskPanic(#[from] JoinError),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let body = Json(json!({
            "error": self.to_string(),
        }));

        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

impl From<ConfigError> for ServerError {
    fn from(value: ConfigError) -> Self {
        match value {
            ConfigError::Io(inner) => ServerError::ParseSchema(inner),
            ConfigError::ProjectPath => ServerError::ProjectPath,
            ConfigError::ParseSchema(inner) => ServerError::ParseSchema(inner),
            ConfigError::NodeError(inner) => inner.into(),
            ConfigError::LoadTsConfig(inner) => ServerError::LoadTsConfig(inner),
        }
    }
}

impl From<NodeError> for ServerError {
    fn from(value: NodeError) -> Self {
        match value {
            NodeError::NodeInPath => ServerError::NodeInPath,
            NodeError::OutdatedNode(one, two) => ServerError::OutdatedNode(one, two),
            NodeError::CheckNodeVersion => ServerError::CheckNodeVersion,
        }
    }
}
