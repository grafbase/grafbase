use runtime::error::{PartialErrorCode, PartialGraphqlError};

pub use crate::extension::wit::{Error as GuestError, ErrorResponse};

pub(crate) fn guest_error_as_gql(error: GuestError, code: PartialErrorCode) -> PartialGraphqlError {
    let extensions = error
        .extensions
        .into_iter()
        .map(|(key, value)| {
            let value = serde_json::from_str(&value).unwrap_or(serde_json::Value::String(value));

            (key.into(), value)
        })
        .collect();

    PartialGraphqlError {
        message: error.message.into(),
        code,
        extensions,
    }
}
