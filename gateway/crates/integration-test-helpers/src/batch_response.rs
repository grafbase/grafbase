#[derive(Debug)]
pub struct GraphqlHttpBatchResponse {
    pub status: http::StatusCode,
    pub headers: http::HeaderMap,
    pub body: serde_json::Value,
}

impl GraphqlHttpBatchResponse {
    pub fn into_body(self) -> serde_json::Value {
        self.body
    }
}
