use engine_error::{ErrorCode, GraphqlError};

use crate::{cbor, state::WasiState};

pub use super::grafbase::sdk::error::*;

impl Host for WasiState {}

impl Error {
    pub(crate) fn into_graphql_error(self, code: ErrorCode) -> GraphqlError {
        GraphqlError::new(self.message, code).with_extensions(self.extensions.into_iter().map(|(key, value)| {
            let value: serde_json::Value = cbor::from_slice(&value).unwrap_or_default();
            (key, value)
        }))
    }
}

impl ErrorResponse {
    pub(crate) fn into_graphql_response(self, code: ErrorCode) -> engine_error::ErrorResponse {
        engine_error::ErrorResponse::new(
            http::StatusCode::from_u16(self.status_code).unwrap_or(http::StatusCode::INTERNAL_SERVER_ERROR),
        )
        .with_errors(self.errors.into_iter().map(|error| error.into_graphql_error(code)))
    }
}
