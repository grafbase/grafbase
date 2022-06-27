use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Json;
use common::traits::ToExitCode;
use hyper::Error as HyperError;
use serde_json::json;
use sqlx::Error as SqlxError;
use std::io::Error as IoError;
use std::path::PathBuf;
use thiserror::Error;
use tokio::task::JoinError;

#[derive(Error, Debug)]
pub enum DevServerError {
    /// returned if the current directory path cannot be read
    #[error("could not create path '{0}' for the embedded server files")]
    CreateDir(PathBuf),

    /// returned if any of the embedded worker files cannot be written to disk
    #[error("could not write an embedded server file: {0}")]
    WriteFile(String),

    /// returned if the version of the existing worker files cannot be read
    #[error("could not read the previously extracted embedded file versions")]
    ReadVersion,

    /// returned if a connection to the sqlite database could not be made
    #[error("could not connect to the sqlite database: {0}")]
    ConnectToDatabase(SqlxError),

    /// returned if an sqlite database file cannot be created
    #[error("could not create an sqlite database file: {0}")]
    CreateDatabase(SqlxError),

    /// returned if an sqlite query returns an error
    #[error("could not query the sqlite database: {0}")]
    QueryDatabase(SqlxError),

    /// returned if sqlx returns an unknown error
    #[error("encountered an unknown sqlite error: {0}")]
    UnknownSqliteError(SqlxError),

    /// returned if the sqlite bridge cannot be started
    #[error("the bridge api encountered an error: {0}")]
    BridgeApi(HyperError),

    /// returned if the miniflare command returns an error
    #[error("miniflare encountered an error: {0}")]
    MiniflareCommandError(IoError),

    /// returned if the miniflare command exits unsuccessfully
    #[error("miniflare encountered an error\ncause:\n{0}")]
    MiniflareError(String),

    /// returned if the schema parser command returns an error
    #[error(transparent)]
    SchemaParserError(IoError),

    /// returned if the schema parser command exits unsuccessfully
    #[error("could not parse grafbase/schema.graphql\n{0}")]
    ParseSchema(String),

    /// returned if the user project path is not valid utf-8
    #[error("non utf-8 path used for project")]
    ProjectPath,

    /// returned if the user cache path is not valid utf-8
    #[error("$HOME/.grafbase is a non utf8 path")]
    CachePath,

    /// returned if the `.grafbase` folder cannot be created
    #[error("could not create a project cache directory")]
    CreateCacheDir,

    /// returned if an available port cannot be found for the bridge server
    #[error("could not find an available port for the bridge server")]
    AvailablePort,

    /// returned if a spawned task panics
    #[error(transparent)]
    SpawnedTaskPanic(JoinError),

    /// returned if node is not in the user $PATH
    #[error("Node.js does not seem to be installed")]
    NodeInPath,

    /// returned if the installed version of node is unsupported
    #[error("Node.js version {0} is unsupported")]
    OutdatedNode(String, String),

    /// returned if the installed version of node could not be retreived
    #[error("Could not retrive the installed version of Node.js")]
    CheckNodeVersion,
}

impl ToExitCode for DevServerError {
    fn to_exit_code(&self) -> i32 {
        match &self {
            Self::CreateDir(_)
            | Self::CreateCacheDir
            | Self::WriteFile(_)
            | Self::ReadVersion
            | Self::ParseSchema(_)
            | Self::NodeInPath
            | Self::OutdatedNode(_, _) => exitcode::DATAERR,
            Self::CreateDatabase(_)
            | Self::QueryDatabase(_)
            | Self::BridgeApi(_)
            | Self::ConnectToDatabase(_)
            | Self::UnknownSqliteError(_)
            | Self::MiniflareCommandError(_)
            | Self::MiniflareError(_)
            | Self::SpawnedTaskPanic(_)
            | Self::SchemaParserError(_)
            | Self::CheckNodeVersion => exitcode::SOFTWARE,
            Self::ProjectPath | Self::CachePath => exitcode::CANTCREAT,
            Self::AvailablePort => exitcode::UNAVAILABLE,
        }
    }
}

impl From<SqlxError> for DevServerError {
    fn from(error: SqlxError) -> Self {
        match error {
            SqlxError::RowNotFound
            | SqlxError::TypeNotFound { .. }
            | SqlxError::ColumnNotFound(_)
            | SqlxError::ColumnDecode { .. }
            | SqlxError::ColumnIndexOutOfBounds { .. }
            | SqlxError::Io(_)
            | SqlxError::Decode(_)
            | SqlxError::Database(_) => Self::QueryDatabase(error),

            SqlxError::Configuration(_)
            | SqlxError::Tls(_)
            | SqlxError::PoolTimedOut
            | SqlxError::Protocol(_)
            | SqlxError::PoolClosed
            | SqlxError::WorkerCrashed => Self::ConnectToDatabase(error),

            SqlxError::Migrate(_) => Self::CreateDatabase(error),

            _ => Self::UnknownSqliteError(error),
        }
    }
}

impl From<HyperError> for DevServerError {
    fn from(error: HyperError) -> Self {
        Self::BridgeApi(error)
    }
}

impl IntoResponse for DevServerError {
    fn into_response(self) -> Response {
        let body = Json(json!({
            "error": self.to_string(),
        }));

        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

impl From<JoinError> for DevServerError {
    fn from(error: JoinError) -> Self {
        Self::SpawnedTaskPanic(error)
    }
}
