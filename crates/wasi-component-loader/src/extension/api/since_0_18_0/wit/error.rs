use crate::{cbor, state::WasiState};
use engine_error::{ErrorCode, GraphqlError};

pub use super::grafbase::sdk::error::*;
use super::{headers::HeaderError, http_client::HttpError};

impl Host for WasiState {}

impl From<Error> for crate::Error {
    fn from(value: Error) -> Self {
        let error_0_19_0 = crate::extension::api::since_0_19_0::wit::grafbase::sdk::error::Error::from(value);
        crate::Error::Guest(error_0_19_0)
    }
}

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

impl From<HeaderError> for crate::extension::api::since_0_17_0::wit::headers::HeaderError {
    fn from(value: HeaderError) -> Self {
        match value {
            HeaderError::InvalidSyntax => Self::InvalidSyntax,
            HeaderError::Forbidden => Self::Forbidden,
            HeaderError::Immutable => Self::Immutable,
        }
    }
}

impl From<crate::extension::api::since_0_17_0::wit::headers::HeaderError> for HeaderError {
    fn from(value: crate::extension::api::since_0_17_0::wit::headers::HeaderError) -> Self {
        match value {
            crate::extension::api::since_0_17_0::wit::headers::HeaderError::InvalidSyntax => Self::InvalidSyntax,
            crate::extension::api::since_0_17_0::wit::headers::HeaderError::Forbidden => Self::Forbidden,
            crate::extension::api::since_0_17_0::wit::headers::HeaderError::Immutable => Self::Immutable,
        }
    }
}

impl From<crate::extension::api::since_0_17_0::wit::error::Error> for Error {
    fn from(value: crate::extension::api::since_0_17_0::wit::error::Error) -> Self {
        Self {
            extensions: value.extensions,
            message: value.message,
        }
    }
}

impl From<crate::extension::api::since_0_17_0::wit::http_client::HttpError> for HttpError {
    fn from(value: crate::extension::api::since_0_17_0::wit::http_client::HttpError) -> Self {
        match value {
            crate::extension::api::since_0_17_0::world::HttpError::Timeout => Self::Timeout,
            crate::extension::api::since_0_17_0::world::HttpError::Request(e) => Self::Request(e),
            crate::extension::api::since_0_17_0::world::HttpError::Connect(e) => Self::Connect(e),
        }
    }
}

impl From<HttpError> for crate::extension::api::since_0_17_0::wit::http_client::HttpError {
    fn from(value: HttpError) -> Self {
        match value {
            HttpError::Timeout => Self::Timeout,
            HttpError::Request(e) => Self::Request(e),
            HttpError::Connect(e) => Self::Connect(e),
        }
    }
}
