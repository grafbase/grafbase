use crate::{cbor, state::WasiState};
use engine_error::{ErrorCode, GraphqlError};

pub use super::grafbase::sdk::error::*;
use crate::extension::api::since_0_10_0::world as wit010;

impl Host for WasiState {}

impl Error {
    pub(crate) fn into_graphql_error(self, code: ErrorCode) -> GraphqlError {
        GraphqlError::new(self.message, code).with_extensions(self.extensions.into_iter().map(|(key, value)| {
            let value: serde_json::Value = cbor::from_slice(&value).unwrap_or_default();
            (key, value)
        }))
    }
}

impl From<wit010::Error> for Error {
    fn from(wit010::Error { extensions, message }: wit010::Error) -> Self {
        Error { extensions, message }
    }
}

impl From<wit010::ErrorResponse> for ErrorResponse {
    fn from(wit010::ErrorResponse { status_code, errors }: wit010::ErrorResponse) -> Self {
        ErrorResponse {
            status_code,
            errors: errors.into_iter().map(Error::from).collect(),
            headers: None,
        }
    }
}
