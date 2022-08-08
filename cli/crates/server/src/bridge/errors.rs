use super::sqlite::extended_error_codes;
use super::types::Constraint;
use super::types::Operation;
use super::types::OperationKind;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Json;
use serde::Serialize;
use sqlx::Error as SqlxError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    /// returned to the worker when user input is invalid
    #[error("user error")]
    User(UserError),
    /// used for bugs / logic errors,
    /// the bridge server will panic when this is encountered
    /// and generate a report
    #[error(transparent)]
    LogicError(SqlxError),
}

#[derive(Serialize, Debug)]
pub enum UserError {
    ConstraintViolation(Constraint),
}

impl From<SqlxError> for ApiError {
    fn from(error: SqlxError) -> Self {
        Self::LogicError(error)
    }
}

#[allow(dead_code)]
#[derive(Serialize, Debug)]
pub struct ApiErrorResponse {
    #[serde(flatten)]
    pub kind: UserError,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::User(user_error) => (StatusCode::CONFLICT, Json(user_error)).into_response(),
            ApiError::LogicError(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

impl ApiError {
    pub fn from_error_and_operation(error: SqlxError, operation: Operation) -> Self {
        match (operation.kind, error.as_database_error()) {
            (Some(OperationKind::Constraint(constraint)), Some(db_error)) => match db_error.code().as_deref() {
                Some(extended_error_codes::SQLITE_CONSTRAINT_PRIMARYKEY) => {
                    ApiError::User(UserError::ConstraintViolation(constraint))
                }
                _ => ApiError::LogicError(error),
            },
            _ => ApiError::LogicError(error),
        }
    }
}
