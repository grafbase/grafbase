mod builder;

use std::{
    borrow::Cow,
    future::IntoFuture,
    ops::{Deref, DerefMut},
    str::FromStr,
    sync::Arc,
};

pub use builder::*;
use engine::{BatchRequest, Variables};
use engine_v2::{HttpGraphqlResponse, HttpGraphqlResponseBody};
use futures::{future::BoxFuture, stream::BoxStream, StreamExt, TryStreamExt};
use gateway_core::StreamingFormat;
use headers::HeaderMapExt;
use http::{header::Entry, HeaderName, HeaderValue};
use serde::de::Error;

use crate::{engine_v1::GraphQlRequest, fetch::RecordedSubRequest};

pub struct TestEngineV2 {
    engine: Arc<engine_v2::Engine<TestRuntime>>,
    recorded_subrequests: Arc<crossbeam_queue::SegQueue<RecordedSubRequest>>,
}

impl TestEngineV2 {
    pub fn execute(&self, request: impl Into<GraphQlRequest>) -> ExecutionRequest {
        while self.recorded_subrequests.pop().is_some() {}
        ExecutionRequest {
            request: request.into(),
            headers: Vec::new(),
            engine: Arc::clone(&self.engine),
        }
    }

    pub fn get_recorded_subrequests(&self) -> Vec<RecordedSubRequest> {
        std::iter::from_fn(|| self.recorded_subrequests.pop()).collect()
    }
}

#[must_use]
pub struct ExecutionRequest {
    request: GraphQlRequest,
    #[allow(dead_code)]
    headers: Vec<(String, String)>,
    engine: Arc<engine_v2::Engine<TestRuntime>>,
}

impl ExecutionRequest {
    pub fn by_client(self, name: &'static str, version: &'static str) -> Self {
        self.header("x-grafbase-client-name", name)
            .header("x-grafbase-client-version", version)
    }

    /// Adds a header into the request
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
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
        let mut headers = http::HeaderMap::new();

        for (key, value) in &self.headers {
            let key = HeaderName::from_str(key).unwrap();
            let value = HeaderValue::from_str(value).unwrap();

            if let Entry::Occupied(mut e) = headers.entry(key.clone()) {
                e.append(value);
            } else {
                headers.insert(key, value);
            }
        }

        headers
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

impl DerefMut for GraphqlResponse {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.body
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
