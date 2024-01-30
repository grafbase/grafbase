//! graphql-ws-client <> engine glue code

use runtime::fetch::{FetchError, GraphqlRequest};
use serde_json::json;

pub struct EngineGraphqlClient;

impl graphql_ws_client::graphql::GraphqlClient for EngineGraphqlClient {
    type Response = serde_json::Value;

    type DecodeError = FetchError;

    fn error_response(errors: Vec<serde_json::Value>) -> Result<Self::Response, Self::DecodeError> {
        Ok(json!({"errors": errors}))
    }
}

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
    type GenericResponse = serde_json::Value;

    type Response = serde_json::Value;

    type Error = FetchError;

    fn decode(&self, data: Self::GenericResponse) -> Result<Self::Response, Self::Error> {
        Ok(data)
    }
}

pub struct TokioSpawner(tokio::runtime::Handle);

impl TokioSpawner {
    pub fn new(handle: tokio::runtime::Handle) -> Self {
        TokioSpawner(handle)
    }

    pub fn current() -> Self {
        TokioSpawner::new(tokio::runtime::Handle::current())
    }
}

impl futures_util::task::Spawn for TokioSpawner {
    fn spawn_obj(&self, obj: futures_util::task::FutureObj<'static, ()>) -> Result<(), futures_util::task::SpawnError> {
        self.0.spawn(obj);
        Ok(())
    }
}
