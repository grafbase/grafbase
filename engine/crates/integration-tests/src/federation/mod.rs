mod builder;

use std::{borrow::Cow, collections::HashMap, future::IntoFuture, ops::Deref, sync::Arc};

pub use builder::*;
use engine::{HttpGraphqlRequest, HttpGraphqlResponse, ResponseBody, Variables};
use futures::{future::BoxFuture, stream::BoxStream, StreamExt};
use headers::HeaderMapExt;
use runtime::cache::CacheStatus;
use serde::{de::Error, Serialize};

use crate::engine::GraphQlRequest;

pub struct TestFederationGateway {
    gateway: Arc<engine_v2::Engine>,
}

impl TestFederationGateway {
    pub fn execute(&self, operation: impl Into<GraphQlRequest>) -> ExecutionRequest {
        ExecutionRequest {
            graphql: operation.into(),
            headers: HashMap::new(),
            gateway: Arc::clone(&self.gateway),
        }
    }
}

#[must_use]
pub struct ExecutionRequest {
    graphql: GraphQlRequest,
    #[allow(dead_code)]
    headers: HashMap<String, String>,
    gateway: Arc<engine_v2::Engine>,
}

impl ExecutionRequest {
    /// Adds a header into the request
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    pub fn variables(mut self, variables: impl serde::Serialize) -> Self {
        self.graphql.variables = Some(Variables::from_json(
            serde_json::to_value(variables).expect("variables to be serializable"),
        ));
        self
    }

    pub fn extensions(mut self, extensions: impl serde::Serialize) -> Self {
        self.graphql.extensions =
            serde_json::from_value(serde_json::to_value(extensions).expect("extensions to be serializable"))
                .expect("extensions to be deserializable");
        self
    }

    fn http_headers(&self) -> http::HeaderMap {
        TryFrom::try_from(&self.headers).unwrap()
    }

    pub fn into_stream(self) -> StreamRequest {
        StreamRequest(self)
    }
}

impl IntoFuture for ExecutionRequest {
    type Output = GraphqlResponse;

    type IntoFuture = BoxFuture<'static, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        let headers = self.http_headers();
        let bytes = serde_json::to_vec(&self.graphql).unwrap();
        let ray_id = ulid::Ulid::new().to_string();
        Box::pin(async move {
            let response = self
                .gateway
                .execute(headers, &ray_id, HttpGraphqlRequest::JsonBody(bytes.into()))
                .await;

            GraphqlResponse::try_from(response).unwrap()
        })
    }
}

pub struct StreamRequest(ExecutionRequest);

impl StreamRequest {
    pub async fn collect<B>(self) -> B
    where
        B: Default + Extend<serde_json::Value>,
    {
        self.await.stream.collect().await
    }
}

impl IntoFuture for StreamRequest {
    type Output = GraphqlStreamingResponse;

    type IntoFuture = BoxFuture<'static, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        let headers = self.0.http_headers();
        let bytes = serde_json::to_vec(&self.0.graphql).unwrap();
        let ray_id = ulid::Ulid::new().to_string();
        Box::pin(async move {
            let response = self
                .0
                .gateway
                .execute(headers, &ray_id, HttpGraphqlRequest::JsonBody(bytes.into()))
                .await;
            GraphqlStreamingResponse {
                stream: match response.body {
                    ResponseBody::Bytes(bytes) => Box::pin(futures::stream::once(async move {
                        serde_json::from_slice(bytes.as_ref()).unwrap()
                    })),
                    ResponseBody::Stream(stream) => Box::pin(stream.map(|result| match result {
                        Ok(bytes) => serde_json::from_slice(bytes.as_ref()).unwrap(),
                        Err(message) => serde_json::Value::String(message),
                    })),
                },
                metadata: response.metadata,
                headers: response.headers,
            }
        })
    }
}

pub struct GraphqlStreamingResponse {
    pub stream: BoxStream<'static, serde_json::Value>,
    pub metadata: engine::ExecutionMetadata,
    pub headers: http::HeaderMap,
}

impl TryFrom<HttpGraphqlResponse> for GraphqlResponse {
    type Error = serde_json::Error;

    fn try_from(response: HttpGraphqlResponse) -> Result<Self, Self::Error> {
        Ok(GraphqlResponse {
            gql_response: match response.body {
                ResponseBody::Bytes(bytes) => serde_json::from_slice(bytes.as_ref())?,
                ResponseBody::Stream(_) => return Err(serde_json::Error::custom("Unexpected stream response body"))?,
            },
            metadata: response.metadata,
            headers: response.headers,
        })
    }
}

#[derive(Serialize, Debug)]
pub struct GraphqlResponse {
    #[serde(flatten)]
    gql_response: serde_json::Value,
    #[serde(skip)]
    pub metadata: engine::ExecutionMetadata,
    #[serde(serialize_with = "serialize_headers", skip_serializing_if = "has_not_ignored_header")]
    pub headers: http::HeaderMap,
}

impl std::fmt::Display for GraphqlResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(&self.gql_response).unwrap())
    }
}

fn serialize_headers<S>(headers: &http::HeaderMap, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    headers
        .iter()
        .filter_map(|(name, value)| match name.as_str() {
            "content-length" | "content-type" => None,
            name => Some((name, value.to_str().ok()?)),
        })
        .collect::<HashMap<&str, &str>>()
        .serialize(serializer)
}

fn has_not_ignored_header(headers: &http::HeaderMap) -> bool {
    headers
        .iter()
        .any(|(name, _)| matches!(name.as_str(), "content-length" | "content-type"))
}

impl Deref for GraphqlResponse {
    type Target = serde_json::Value;

    fn deref(&self) -> &Self::Target {
        &self.gql_response
    }
}

impl GraphqlResponse {
    pub fn into_value(self) -> serde_json::Value {
        self.gql_response
    }

    #[track_caller]
    pub fn into_data(self) -> serde_json::Value {
        assert!(self.errors().is_empty(), "{self:#?}");

        match self.gql_response {
            serde_json::Value::Object(mut value) => value.remove("data"),
            _ => None,
        }
        .unwrap_or_default()
    }

    pub fn errors(&self) -> Cow<'_, Vec<serde_json::Value>> {
        self.gql_response["errors"]
            .as_array()
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Owned(Vec::new()))
    }

    pub fn cache_control(&self) -> Option<headers::CacheControl> {
        self.headers.typed_get()
    }

    pub fn cache_status(&self) -> Option<CacheStatus> {
        self.headers.typed_get()
    }
}
