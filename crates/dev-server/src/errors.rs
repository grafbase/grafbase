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

    /// returned if the miniflare thread returns an error
    #[error(transparent)]
    MiniflareError(IoError),

    /// returned if an available port cannot be found for the bridge server
    #[error("could not find an available port for the bridge server")]
    AvailablePort,
}

impl ToExitCode for DevServerError {
    fn to_exit_code(&self) -> i32 {
        match &self {
            Self::CreateDir(_) | Self::WriteFile(_) | Self::ReadVersion => exitcode::DATAERR,
            Self::CreateDatabase(_)
            | Self::QueryDatabase(_)
            | Self::BridgeApi(_)
            | Self::ConnectToDatabase(_)
            | Self::UnknownSqliteError(_)
            | Self::MiniflareError(_) => exitcode::SOFTWARE,
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
