mod stream;

use std::{
    borrow::Cow,
    collections::HashMap,
    future::IntoFuture,
    ops::{Deref, DerefMut},
};

use axum::body::Body;
use bytes::Bytes;
use futures::future::BoxFuture;
use http_body_util::BodyExt;
use serde::{ser::SerializeMap, Deserialize};
pub use stream::*;
use tower::ServiceExt;

#[must_use]
pub struct TestRequest {
    pub(super) router: axum::Router,
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
        self.body.variables = Some(serde_json::to_value(variables).expect("variables to be serializable"));
        self
    }

    pub fn extensions(mut self, extensions: impl serde::Serialize) -> Self {
        self.body.extensions =
            serde_json::from_value(serde_json::to_value(extensions).expect("extensions to be serializable"))
                .expect("extensions to be deserializable");
        self
    }

    pub fn operation_name(mut self, name: impl Into<String>) -> Self {
        self.body.operation_name = Some(name.into());
        self
    }

    pub fn into_multipart_stream(self) -> MultipartStreamRequest {
        MultipartStreamRequest(self)
    }

    pub fn into_sse_stream(self) -> SseStreamRequest {
        SseStreamRequest(self)
    }

    fn into_router_and_request(self) -> (axum::Router, http::Request<Body>) {
        let Self {
            router,
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
                router,
                http::Request::from_parts(parts, Body::from(Bytes::from_static(b""))),
            )
        } else {
            parts
                .headers
                .entry(http::header::CONTENT_TYPE)
                .or_insert(http::HeaderValue::from_static("application/json"));
            let body = serde_json::to_vec(&body).unwrap();
            (router, http::Request::from_parts(parts, Body::from(Bytes::from(body))))
        }
    }
}

impl IntoFuture for TestRequest {
    type Output = GraphqlResponse;

    type IntoFuture = BoxFuture<'static, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let (router, request) = self.into_router_and_request();
            let (parts, body) = router.oneshot(request).await.unwrap().into_parts();
            let bytes = body.collect().await.unwrap().to_bytes();
            http::Response::from_parts(parts, bytes).try_into().unwrap()
        })
    }
}

#[derive(serde::Serialize)]
pub struct GraphQlRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "operationName")]
    pub operation_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<RequestExtensions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc_id: Option<String>,
}

impl GraphQlRequest {
    pub fn into_query_params(self) -> impl serde::Serialize {
        QueryParams(self)
    }
}

impl<'a> From<&'a str> for GraphQlRequest {
    fn from(value: &'a str) -> Self {
        value.to_string().into()
    }
}

impl<'a> From<Option<&'a str>> for GraphQlRequest {
    fn from(value: Option<&'a str>) -> Self {
        value.map(|s| s.to_string()).into()
    }
}

impl From<String> for GraphQlRequest {
    fn from(query: String) -> Self {
        Some(query).into()
    }
}

impl From<Option<String>> for GraphQlRequest {
    fn from(query: Option<String>) -> Self {
        Self {
            query,
            operation_name: None,
            variables: None,
            extensions: None,
            doc_id: None,
        }
    }
}

impl<T, V> From<cynic::Operation<T, V>> for GraphQlRequest
where
    V: serde::Serialize,
{
    fn from(operation: cynic::Operation<T, V>) -> Self {
        GraphQlRequest {
            query: Some(operation.query),
            variables: Some(serde_json::from_value(serde_json::to_value(operation.variables).unwrap()).unwrap()),
            operation_name: operation.operation_name.map(|name| name.to_string()),
            extensions: None,
            doc_id: None,
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestExtensions {
    #[serde(default)]
    pub persisted_query: Option<PersistedQueryRequestExtension>,
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

#[serde_with::serde_as]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedQueryRequestExtension {
    pub version: u32,
    #[serde_as(as = "serde_with::hex::Hex")]
    pub sha256_hash: Vec<u8>,
}

struct QueryParams(GraphQlRequest);

impl serde::Serialize for QueryParams {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("query", &self.0.query)?;

        if let Some(doc_id) = &self.0.doc_id {
            map.serialize_entry("doc_id", doc_id)?;
        }

        if let Some(operation_name) = &self.0.operation_name {
            map.serialize_entry("operation_name", operation_name)?;
        }

        if let Some(variables) = &self.0.variables {
            map.serialize_entry(
                "variables",
                &serde_json::to_string(variables).map_err(serde::ser::Error::custom)?,
            )?;
        }

        if let Some(extensions) = &self.0.extensions {
            map.serialize_entry(
                "extensions",
                &serde_json::to_string(extensions).map_err(serde::ser::Error::custom)?,
            )?;
        }

        map.end()
    }
}

#[derive(serde::Serialize, Debug, Deserialize)]
pub struct GraphqlResponse {
    #[serde(skip)]
    pub status: http::StatusCode,
    #[serde(skip)]
    pub headers: http::HeaderMap,
    #[serde(flatten)]
    pub body: serde_json::Value,
}

impl TryFrom<http::Response<Bytes>> for GraphqlResponse {
    type Error = serde_json::Error;

    fn try_from(response: http::Response<Bytes>) -> Result<Self, Self::Error> {
        let (parts, body) = response.into_parts();

        Ok(GraphqlResponse {
            status: parts.status,
            body: serde_json::from_slice(body.as_ref())?,
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
