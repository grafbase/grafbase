use crate::{cbor, state::WasiState};
use engine_error::{ErrorCode, GraphqlError};

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

impl From<crate::extension::api::since_0_9_0::wit::error::Error> for Error {
    fn from(
        crate::extension::api::since_0_10_0::world::Error { extensions, message }: crate::extension::api::since_0_9_0::wit::error::Error,
    ) -> Self {
        Error { extensions, message }
    }
}

impl From<crate::extension::api::since_0_9_0::wit::error::ErrorResponse> for ErrorResponse {
    fn from(
        crate::extension::api::since_0_10_0::world::ErrorResponse { status_code, errors }: crate::extension::api::since_0_9_0::wit::error::ErrorResponse,
    ) -> Self {
        ErrorResponse {
            status_code,
            errors: errors.into_iter().map(Error::from).collect(),
            headers: None,
        }
    }
}

impl From<crate::extension::api::since_0_9_0::wit::error::Error> for crate::Error {
    fn from(value: crate::extension::api::since_0_9_0::wit::error::Error) -> Self {
        Error::from(value).into()
    }
}

impl From<Error> for crate::Error {
    fn from(value: Error) -> Self {
        let error_0_18_0 = crate::extension::api::since_0_18_0::wit::error::Error::from(value);
        crate::Error::Guest(error_0_18_0)
    }
}
