use engine::{ErrorCode, GraphqlError};
use wasi_component_loader::GuestError;

pub mod hooks;

fn guest_error_as_gql(error: GuestError, code: ErrorCode) -> GraphqlError {
    GraphqlError::new(error.message, code).with_extensions(error.extensions.into_iter().map(|(key, value)| {
        let value = String::from_utf8_lossy(&value).into_owned();
        let value = serde_json::from_str(&value).unwrap_or(serde_json::Value::String(value));
        (key, value)
    }))
}
