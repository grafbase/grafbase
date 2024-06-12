mod builder;

use std::{borrow::Cow, collections::HashMap, future::IntoFuture, ops::Deref, sync::Arc};

pub use builder::*;
use engine::{BatchRequest, Variables};
use engine_v2::{HttpGraphqlResponse, HttpGraphqlResponseBody};
use futures::{future::BoxFuture, stream::BoxStream, StreamExt, TryStreamExt};
use gateway_core::StreamingFormat;
use headers::HeaderMapExt;
use serde::de::Error;

use crate::engine_v1::GraphQlRequest;

pub struct TestFederationEngine {
    engine: Arc<engine_v2::Engine>,
}

impl TestFederationEngine {
    pub fn new(engine: Arc<engine_v2::Engine>) -> Self {
        TestFederationEngine { engine }
    }

    pub fn execute(&self, request: impl Into<GraphQlRequest>) -> ExecutionRequest {
        ExecutionRequest {
            request: request.into(),
            headers: HashMap::new(),
            engine: Arc::clone(&self.engine),
        }
    }
}

#[must_use]
pub struct ExecutionRequest {
    request: GraphQlRequest,
    #[allow(dead_code)]
    headers: HashMap<String, String>,
    engine: Arc<engine_v2::Engine>,
}

impl ExecutionRequest {
    /// Adds a header into the request
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    pub fn variables(mut self, variables: impl serde::Serialize) -> Self {
        self.request.variables = Some(Variables::from_json(
            serde_json::to_value(variables).expect("variables to be serializable"),
        ));
        self
    }

    pub fn extensions(mut self, extensions: impl serde::Serialize) -> Self {
        self.request.extensions =
            serde_json::from_value(serde_json::to_value(extensions).expect("extensions to be serializable"))
                .expect("extensions to be deserializable");
        self
    }

    fn http_headers(&self) -> http::HeaderMap {
        TryFrom::try_from(&self.headers).unwrap()
    }

    pub fn into_multipart_stream(self) -> MultipartStreamRequest {
        MultipartStreamRequest(self)
    }
}

impl IntoFuture for ExecutionRequest {
    type Output = GraphqlResponse;

    type IntoFuture = BoxFuture<'static, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        let headers = self.http_headers();
        let request = BatchRequest::Single(self.request.into_engine_request());
        Box::pin(async move { self.engine.execute(headers, request).await.try_into().unwrap() })
    }
}

pub struct MultipartStreamRequest(ExecutionRequest);

impl MultipartStreamRequest {
    pub async fn collect<B>(self) -> B
    where
        B: Default + Extend<serde_json::Value>,
    {
        self.await.stream.collect().await
    }
}

impl IntoFuture for MultipartStreamRequest {
    type Output = GraphqlStreamingResponse;

    type IntoFuture = BoxFuture<'static, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        let mut headers = self.0.http_headers();
        headers.typed_insert(StreamingFormat::IncrementalDelivery);
        let request = BatchRequest::Single(self.0.request.into_engine_request());
        Box::pin(async move {
            let response = self.0.engine.execute(headers, request).await;
            let stream = multipart_stream::parse(response.body.into_stream().map_ok(Into::into), "-")
                .map(|result| serde_json::from_slice(&result.unwrap().body).unwrap());
            GraphqlStreamingResponse {
                stream: Box::pin(stream),
                headers: response.headers,
            }
        })
    }
}

pub struct GraphqlStreamingResponse {
    pub stream: BoxStream<'static, serde_json::Value>,
    pub headers: http::HeaderMap,
}

#[derive(serde::Serialize, Debug)]
pub struct GraphqlResponse {
    #[serde(flatten)]
    pub body: serde_json::Value,
    #[serde(skip)]
    pub headers: http::HeaderMap,
}

impl TryFrom<HttpGraphqlResponse> for GraphqlResponse {
    type Error = serde_json::Error;

    fn try_from(response: HttpGraphqlResponse) -> Result<Self, Self::Error> {
        Ok(GraphqlResponse {
            body: match response.body {
                HttpGraphqlResponseBody::Bytes(bytes) => serde_json::from_slice(bytes.as_ref())?,
                HttpGraphqlResponseBody::Stream(_) => {
                    return Err(serde_json::Error::custom("Unexpected stream response body"))?
                }
            },
            headers: response.headers,
        })
    }
}

impl std::fmt::Display for GraphqlResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(&self.body).unwrap())
    }
}

impl Deref for GraphqlResponse {
    type Target = serde_json::Value;

    fn deref(&self) -> &Self::Target {
        &self.body
    }
}

impl GraphqlResponse {
    pub fn into_value(self) -> serde_json::Value {
        self.body
    }

    #[track_caller]
    pub fn into_data(self) -> serde_json::Value {
        assert!(self.errors().is_empty(), "{self:#?}");

        match self.body {
            serde_json::Value::Object(mut value) => value.remove("data"),
            _ => None,
        }
        .unwrap_or_default()
    }

    pub fn errors(&self) -> Cow<'_, Vec<serde_json::Value>> {
        self.body["errors"]
            .as_array()
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Owned(Vec::new()))
    }
}
