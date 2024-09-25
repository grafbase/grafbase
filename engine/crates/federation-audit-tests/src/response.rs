use serde::Serialize;

use crate::audit_server::ExpectedResponse;

/// Utility struct for comparing responses with ExpectedResponse
#[derive(PartialEq, Debug, Serialize)]
pub struct Response<'a> {
    pub data: serde_json::Value,
    pub errors: &'a [serde_json::Value],
}

impl PartialEq<ExpectedResponse> for Response<'_> {
    fn eq(&self, other: &ExpectedResponse) -> bool {
        self.data == other.data && self.errors.is_empty() != other.errors
    }
}
