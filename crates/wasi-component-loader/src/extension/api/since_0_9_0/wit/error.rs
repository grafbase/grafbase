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

impl From<ErrorResponse> for crate::ErrorResponse {
    fn from(value: crate::extension::api::since_0_9_0::wit::error::ErrorResponse) -> Self {
        crate::ErrorResponse::Guest {
            status_code: http::StatusCode::from_u16(value.status_code)
                .unwrap_or(http::StatusCode::INTERNAL_SERVER_ERROR),
            errors: value.errors.into_iter().map(From::from).collect(),
            headers: Default::default(),
        }
    }
}
