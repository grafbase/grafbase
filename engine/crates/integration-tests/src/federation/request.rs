mod stream;

use std::{
    borrow::Cow,
    future::IntoFuture,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use bytes::Bytes;
use engine::Variables;
use engine_v2::Body;
use futures::future::BoxFuture;
use serde::de::Error;
pub use stream::*;

use crate::engine_v1::GraphQlRequest;

use super::TestRuntime;

type RequestBodyFut = BoxFuture<'static, Result<Bytes, (http::StatusCode, String)>>;

#[must_use]
pub struct TestRequest {
    pub(super) engine: Arc<engine_v2::Engine<TestRuntime>>,
    pub(super) parts: http::request::Parts,
    pub(super) body: GraphQlRequest,
}

impl TestRequest {
    pub fn by_client(self, name: &'static str, version: &'static str) -> Self {
        self.header("x-grafbase-client-name", name)
            .header("x-grafbase-client-version", version)
    }

    pub fn header<Name, Value>(mut self, name: Name, value: Value) -> Self
    where
        Name: TryInto<http::HeaderName, Error: std::fmt::Debug>,
        Value: TryInto<http::HeaderValue, Error: std::fmt::Debug>,
    {
        self.parts
            .headers
            .insert(name.try_into().unwrap(), value.try_into().unwrap());
        self
    }

    pub fn header_append<Name, Value>(mut self, name: Name, value: Value) -> Self
    where
        Name: TryInto<http::HeaderName, Error: std::fmt::Debug>,
        Value: TryInto<http::HeaderValue, Error: std::fmt::Debug>,
    {
        self.parts
            .headers
            .append(name.try_into().unwrap(), value.try_into().unwrap());
        self
    }

    pub fn variables(mut self, variables: impl serde::Serialize) -> Self {
        self.body.variables = Some(Variables::from_json(
            serde_json::to_value(variables).expect("variables to be serializable"),
        ));
        self
    }

    pub fn extensions(mut self, extensions: impl serde::Serialize) -> Self {
        self.body.extensions =
            serde_json::from_value(serde_json::to_value(extensions).expect("extensions to be serializable"))
                .expect("extensions to be deserializable");
        self
    }

    pub fn into_multipart_stream(self) -> MultipartStreamRequest {
        MultipartStreamRequest(self)
    }

    pub fn into_sse_stream(self) -> SseStreamRequest {
        SseStreamRequest(self)
    }

    fn into_engine_and_request(self) -> (Arc<engine_v2::Engine<TestRuntime>>, http::Request<RequestBodyFut>) {
        let Self {
            engine,
            mut parts,
            body,
        } = self;
        if parts.method == http::Method::GET {
            parts.uri = http::uri::Builder::from(std::mem::take(&mut parts.uri))
                .path_and_query(format!(
                    "/graphql?{}",
                    serde_urlencoded::to_string(body.into_query_params()).unwrap()
                ))
                .build()
                .unwrap();
            (
                engine,
                http::Request::from_parts(parts, Box::pin(async { Ok(Bytes::from_static(b"")) })),
            )
        } else {
            parts
                .headers
                .entry(http::header::CONTENT_TYPE)
                .or_insert(http::HeaderValue::from_static("application/json"));
            let body = serde_json::to_vec(&body).unwrap();
            (
                engine,
                http::Request::from_parts(parts, Box::pin(async move { Ok(Bytes::from(body)) })),
            )
        }
    }
}

impl IntoFuture for TestRequest {
    type Output = GraphqlResponse;

    type IntoFuture = BoxFuture<'static, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let (engine, mut request) = self.into_engine_and_request();
            request
                .headers_mut()
                .entry(http::header::ACCEPT)
                .or_insert(http::HeaderValue::from_static("application/json"));
            engine.execute(request).await.try_into().unwrap()
        })
    }
}

#[derive(serde::Serialize, Debug)]
pub struct GraphqlResponse {
    #[serde(skip)]
    pub status: http::StatusCode,
    #[serde(skip)]
    pub headers: http::HeaderMap,
    #[serde(flatten)]
    pub body: serde_json::Value,
}

impl TryFrom<http::Response<Body>> for GraphqlResponse {
    type Error = serde_json::Error;

    fn try_from(response: http::Response<Body>) -> Result<Self, Self::Error> {
        let (parts, body) = response.into_parts();
        Ok(GraphqlResponse {
            status: parts.status,
            body: match body {
                Body::Bytes(bytes) => serde_json::from_slice(bytes.as_ref())?,
                Body::Stream(_) => return Err(serde_json::Error::custom("Unexpected stream response body"))?,
            },
            headers: parts.headers,
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
