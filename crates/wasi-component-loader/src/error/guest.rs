use runtime::error::{PartialErrorCode, PartialGraphqlError};

pub use crate::extension::wit::{Error as GuestError, ErrorResponse};

pub(crate) fn guest_error_as_gql(error: GuestError, code: PartialErrorCode) -> PartialGraphqlError {
    let extensions = error
        .extensions
        .into_iter()
        .map(|(key, value)| {
            let value = minicbor_serde::from_slice(&value).unwrap_or_default();
            (key.into(), value)
        })
        .collect();

    PartialGraphqlError {
        message: error.message.into(),
        code,
        extensions,
    }
}
