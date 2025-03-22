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
