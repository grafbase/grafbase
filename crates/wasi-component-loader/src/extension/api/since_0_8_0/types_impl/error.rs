use runtime::error::{PartialErrorCode, PartialGraphqlError};

use crate::{cbor, extension::api::since_0_8_0::wit::grafbase::sdk::types};

impl types::Error {
    pub(crate) fn into_graphql_error(self, code: PartialErrorCode) -> PartialGraphqlError {
        let extensions = self
            .extensions
            .into_iter()
            .map(|(key, value)| {
                let value = cbor::from_slice(&value).unwrap_or_default();
                (key.into(), value)
            })
            .collect();

        PartialGraphqlError {
            message: self.message.into(),
            code,
            extensions,
        }
    }
}
