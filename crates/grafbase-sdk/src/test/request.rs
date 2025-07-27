use std::{
    borrow::Cow,
    ops::{Deref, DerefMut},
};

use bytes::Bytes;
use futures_util::{StreamExt as _, stream::BoxStream};
use http::HeaderValue;
use http_body_util::BodyExt as _;
use serde::de::DeserializeOwned;

/// Represents a GraphQL request.
pub struct GraphqlRequest {
    pub(super) builder: reqwest::RequestBuilder,
    pub(super) body: Body,
}

impl GraphqlRequest {
    /// Add a header to the request.
    pub fn header<Name, Value>(mut self, name: Name, value: Value) -> Self
    where
        Name: TryInto<http::HeaderName, Error: std::fmt::Debug>,
        Value: TryInto<http::HeaderValue, Error: std::fmt::Debug>,
    {
        self.builder = self.builder.header(name.try_into().unwrap(), value.try_into().unwrap());
        self
    }

    /// Add a set of Headers to the existing ones on this Request.
    ///
    /// The headers will be merged in to any already set.
    pub fn headers(mut self, headers: http::HeaderMap) -> Self {
        self.builder = self.builder.headers(headers);
        self
    }

    /// Add the GraphQL variables to the request.
    pub fn variables(mut self, variables: impl serde::Serialize) -> Self {
        self.body.variables = Some(serde_json::to_value(variables).expect("variables to be serializable"));
        self
    }

    /// Send the GraphQL request to the gateway
    pub async fn send(self) -> GraphqlResponse {
        let response = self
            .builder
            .header(http::header::ACCEPT, "application/json")
            .json(&self.body)
            .send()
            .await
            .expect("Request suceeded");
        let (parts, body) = http::Response::from(response).into_parts();
        let bytes = body.collect().await.expect("Could retrieve response body").to_bytes();
        http::Response::from_parts(parts, bytes).try_into().unwrap()
    }

    /// Send the GraphQL request to the gateway and return a streaming response through a
    /// websocket.
    pub async fn ws_stream(self) -> GraphqlStreamingResponse {
        use async_tungstenite::tungstenite::client::IntoClientRequest as _;
        use futures_util::StreamExt;

        let mut req = self.builder.build().expect("Valid request");
        req.url_mut().set_scheme("ws").expect("Valid URL scheme");
        req.url_mut().set_path("/ws");
        let (parts, _) = http::Request::try_from(req).expect("Valid HTTP request").into_parts();

        let mut request = parts.uri.into_client_request().unwrap();

        request.headers_mut().extend(parts.headers);
        request.headers_mut().insert(
            http::header::SEC_WEBSOCKET_PROTOCOL,
            HeaderValue::from_str("graphql-transport-ws").unwrap(),
        );

        let (connection, response) = async_tungstenite::tokio::connect_async(request)
            .await
            .expect("Request suceeded");
        let (parts, _) = response.into_parts();

        let (client, actor) = graphql_ws_client::Client::build(connection)
            .await
            .expect("Client build succeeded");

        tokio::spawn(actor.into_future());

        let stream: BoxStream<'_, _> = Box::pin(
            client
                .subscribe(self.body)
                .await
                .expect("Subscription succeeded")
                .map(move |item| item.unwrap()),
        );

        GraphqlStreamingResponse {
            status: parts.status,
            headers: parts.headers,
            stream,
        }
    }
}

/// Represents the body of a GraphQL request.
#[derive(serde::Serialize)]
pub struct Body {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) variables: Option<serde_json::Value>,
}

impl<'a> From<&'a str> for Body {
    fn from(value: &'a str) -> Self {
        value.to_string().into()
    }
}

impl<'a> From<&'a String> for Body {
    fn from(value: &'a String) -> Self {
        value.clone().into()
    }
}

impl From<String> for Body {
    fn from(query: String) -> Self {
        Body {
            query: Some(query),
            variables: None,
        }
    }
}

/// Represents a GraphQL response.
#[derive(serde::Serialize, Debug, serde::Deserialize)]
pub struct GraphqlResponse {
    /// The HTTP status code of the response.
    #[serde(skip)]
    status: http::StatusCode,
    /// The HTTP headers of the response.
    #[serde(skip)]
    headers: http::HeaderMap,
    /// The body of the response, which contains the GraphQL data.
    #[serde(flatten)]
    body: serde_json::Value,
}

impl TryFrom<http::Response<Bytes>> for GraphqlResponse {
    type Error = serde_json::Error;

    fn try_from(response: http::Response<Bytes>) -> Result<Self, Self::Error> {
        let (parts, body) = response.into_parts();

        Ok(GraphqlResponse {
            status: parts.status,
            body: serde_json::from_slice(body.as_ref())
                .unwrap_or_else(|err| serde_json::Value::String(format!("Could not deserialize JSON data: {err}"))),
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
    /// Returns the HTTP status code of the response.
    pub fn status(&self) -> http::StatusCode {
        self.status
    }

    /// Returns the HTTP headers of the response.
    pub fn headers(&self) -> &http::HeaderMap {
        &self.headers
    }

    /// Consumes the response and returns the body as a JSON value.
    pub fn into_body(self) -> serde_json::Value {
        self.body
    }

    /// Deserializes the response body
    pub fn deserialize<T: DeserializeOwned>(self) -> anyhow::Result<T> {
        serde_json::from_value(self.body).map_err(Into::into)
    }

    /// Extracts the `data` field from the response body, if it exists.
    #[track_caller]
    pub fn into_data(self) -> serde_json::Value {
        assert!(self.errors().is_empty(), "{self:#?}");

        match self.body {
            serde_json::Value::Object(mut value) => value.remove("data"),
            _ => None,
        }
        .unwrap_or_default()
    }

    /// Returns the `errors` field from the response body, if it exists.
    pub fn errors(&self) -> Cow<'_, Vec<serde_json::Value>> {
        self.body["errors"]
            .as_array()
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Owned(Vec::new()))
    }
}

/// Represents a GraphQL subscription response.
pub struct GraphqlStreamingResponse {
    /// The HTTP status code of the response.
    status: http::StatusCode,
    /// The HTTP headers of the response.
    headers: http::HeaderMap,
    /// The stream of messages from the subscription.
    stream: BoxStream<'static, serde_json::Value>,
}

impl std::ops::Deref for GraphqlStreamingResponse {
    type Target = BoxStream<'static, serde_json::Value>;
    fn deref(&self) -> &Self::Target {
        &self.stream
    }
}

impl std::ops::DerefMut for GraphqlStreamingResponse {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stream
    }
}

impl GraphqlStreamingResponse {
    /// Returns the HTTP status code of the response.
    pub fn status(&self) -> http::StatusCode {
        self.status
    }

    /// Returns the HTTP headers of the response.
    pub fn headers(&self) -> &http::HeaderMap {
        &self.headers
    }

    /// Consumes the response and returns the underlying stream.
    pub fn into_stream(self) -> BoxStream<'static, serde_json::Value> {
        self.stream
    }

    /// Consumes the response and returns the first `n` messages.
    pub async fn take(self, n: usize) -> GraphqlCollectedStreamingResponse {
        let messages = self.stream.take(n).collect().await;
        GraphqlCollectedStreamingResponse {
            status: self.status,
            headers: self.headers,
            messages,
        }
    }

    /// Collect all messages from the subscription stream.
    pub async fn collect(self) -> GraphqlCollectedStreamingResponse {
        let messages = self.stream.collect().await;
        GraphqlCollectedStreamingResponse {
            status: self.status,
            headers: self.headers,
            messages,
        }
    }
}

/// Represents a collected GraphQL subscription response.
#[derive(Debug)]
pub struct GraphqlCollectedStreamingResponse {
    /// The HTTP status code of the response.
    status: http::StatusCode,
    /// The HTTP headers of the response.
    headers: http::HeaderMap,
    /// The collected messages from the subscription.
    messages: Vec<serde_json::Value>,
}

impl GraphqlCollectedStreamingResponse {
    /// Returns the HTTP status code of the response.
    pub fn status(&self) -> http::StatusCode {
        self.status
    }
    /// Returns the HTTP headers of the response.
    pub fn headers(&self) -> &http::HeaderMap {
        &self.headers
    }
    /// Returns the collected messages from the subscription.
    pub fn messages(&self) -> &Vec<serde_json::Value> {
        &self.messages
    }
    /// Consumes the response and returns the collected messages.
    pub fn into_messages(self) -> Vec<serde_json::Value> {
        self.messages
    }
}

impl graphql_ws_client::graphql::GraphqlOperation for Body {
    type Response = serde_json::Value;
    type Error = serde_json::Error;

    fn decode(&self, data: serde_json::Value) -> Result<Self::Response, Self::Error> {
        Ok(data)
    }
}

impl serde::Serialize for GraphqlCollectedStreamingResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.messages.serialize(serializer)
    }
}

pub struct IntrospectionRequest(pub(super) GraphqlRequest);

impl IntrospectionRequest {
    /// Add a header to the request.
    pub fn header<Name, Value>(mut self, name: Name, value: Value) -> Self
    where
        Name: TryInto<http::HeaderName, Error: std::fmt::Debug>,
        Value: TryInto<http::HeaderValue, Error: std::fmt::Debug>,
    {
        self.0 = self.0.header(name, value);
        self
    }

    /// Add a set of Headers to the existing ones on this Request.
    ///
    /// The headers will be merged in to any already set.
    pub fn headers(mut self, headers: http::HeaderMap) -> Self {
        self.0 = self.0.headers(headers);
        self
    }

    /// Send the GraphQL request to the gateway
    pub async fn send(self) -> String {
        let response = self.0.send().await;
        serde_json::from_value::<cynic_introspection::IntrospectionQuery>(response.into_data())
            .expect("valid response")
            .into_schema()
            .expect("valid schema")
            .to_sdl()
    }
}
