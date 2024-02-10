//! graphql-ws-client <> engine glue code

use runtime::fetch::{FetchError, GraphqlRequest};

#[derive(serde::Serialize)]
pub struct StreamingRequest {
    query: String,
    variables: serde_json::Value,
}

impl From<GraphqlRequest<'_>> for StreamingRequest {
    fn from(value: GraphqlRequest<'_>) -> Self {
        StreamingRequest {
            query: value.query.to_string(),
            variables: value.variables,
        }
    }
}

impl graphql_ws_client::graphql::GraphqlOperation for StreamingRequest {
    type Response = serde_json::Value;

    type Error = FetchError;

    fn decode(&self, data: serde_json::Value) -> Result<Self::Response, Self::Error> {
        Ok(data)
    }
}
