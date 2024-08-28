use std::future::IntoFuture;

use futures_lite::{future, FutureExt};
use serde::{Deserialize, Serialize};

use crate::GraphqlHttpResponse;

#[must_use]
pub struct TestRequest {
    pub(super) client: reqwest::Client,
    pub(super) parts: http::request::Parts,
    pub(super) body: GraphQlRequestBody,
}

#[derive(Serialize, Deserialize)]
pub struct GraphQlRequestBody {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    operation_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extensions: Option<serde_json::Value>,
}

impl TestRequest {
    pub fn use_http_get(mut self) -> Self {
        self.parts.method = http::Method::GET;
        self
    }

    pub fn operation_name(mut self, name: &str) -> Self {
        self.body.operation_name = Some(name.into());
        self
    }

    pub fn bearer(self, token: &str) -> Self {
        self.header("Authorization", format!("Bearer {token}"))
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
}

impl IntoFuture for TestRequest {
    type Output = GraphqlHttpResponse;

    type IntoFuture = future::Boxed<Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        async move {
            let http::request::Parts {
                method, uri, headers, ..
            } = self.parts;

            let response = self
                .client
                .request(method, uri.to_string())
                .headers(headers)
                .json(&self.body)
                .send()
                .await
                .expect("http request to succeed");

            GraphqlHttpResponse {
                status: response.status(),
                headers: response.headers().clone(),
                body: response.json().await.expect("a json response"),
            }
        }
        .boxed()
    }
}

impl From<&str> for GraphQlRequestBody {
    fn from(val: &str) -> Self {
        GraphQlRequestBody {
            query: val.into(),
            variables: None,
            operation_name: None,
            extensions: None,
        }
    }
}

impl From<String> for GraphQlRequestBody {
    fn from(query: String) -> Self {
        GraphQlRequestBody {
            query,
            variables: None,
            operation_name: None,
            extensions: None,
        }
    }
}

impl From<&String> for GraphQlRequestBody {
    fn from(query: &String) -> Self {
        GraphQlRequestBody {
            query: query.to_owned(),
            variables: None,
            operation_name: None,
            extensions: None,
        }
    }
}
