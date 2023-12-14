#![allow(dead_code)]
use reqwest::{header::HeaderMap, StatusCode};
use serde_json::json;
use std::{
    future::{Future, IntoFuture},
    marker::PhantomData,
    pin::Pin,
    time::{Duration, SystemTime},
};
use tokio::time::sleep;

use crate::utils::consts::INTROSPECTION_QUERY;

use super::environment::CommandHandles;

pub struct AsyncClient {
    endpoint: String,
    playground_endpoint: String,
    headers: HeaderMap,
    client: reqwest::Client,
    snapshot: Option<String>,
    commands: CommandHandles,
}

impl AsyncClient {
    pub fn new(endpoint: String, playground_endpoint: String, commands: CommandHandles) -> Self {
        Self {
            endpoint,
            playground_endpoint,
            client: reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(1))
                .timeout(Duration::from_secs(20))
                .build()
                .unwrap(),
            snapshot: None,
            headers: HeaderMap::new(),
            commands,
        }
    }

    pub fn with_api_key(self) -> Self {
        self.with_header("x-api-key", "any")
    }

    pub fn with_header(mut self, key: &'static str, value: &str) -> Self {
        self.headers.insert(key, value.parse().unwrap());
        self
    }

    pub fn with_cleared_headers(mut self) -> Self {
        self.headers.clear();
        self
    }

    // TODO: update this one as well...
    pub fn gql<Response>(&self, query: impl Into<String>) -> GqlRequestBuilder<Response>
    where
        Response: serde::de::DeserializeOwned + 'static,
    {
        let reqwest_builder = self.client.post(&self.endpoint).headers(self.headers.clone());

        GqlRequestBuilder {
            query: query.into(),
            variables: None,
            phantom: PhantomData,
            reqwest_builder,
        }
    }

    async fn introspect(&self) -> String {
        self.client
            .post(&self.endpoint)
            .headers(self.headers.clone())
            .body(json!({"operationName":"IntrospectionQuery", "query": INTROSPECTION_QUERY}).to_string())
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap()
    }

    async fn safe_introspect(&self) -> Option<String> {
        if let Ok(response) = self
            .client
            .post(&self.endpoint)
            .headers(self.headers.clone())
            .body(json!({"operationName":"IntrospectionQuery", "query": INTROSPECTION_QUERY}).to_string())
            .send()
            .await
        {
            if response.status() != StatusCode::SERVICE_UNAVAILABLE {
                if let Ok(text) = response.text().await {
                    return Some(text);
                }
            }
        }

        None
    }

    /// # Panics
    ///
    /// panics if the set timeout is reached
    pub async fn poll_endpoint(&self, timeout_secs: u64, interval_millis: u64) {
        let start = SystemTime::now();

        loop {
            assert!(
                self.commands.still_running(),
                "all commands terminated, polling is unlikely to succeed"
            );

            let valid_response = self
                .client
                .head(&self.endpoint)
                .send()
                .await
                .is_ok_and(|response| response.status() != StatusCode::SERVICE_UNAVAILABLE);

            if valid_response {
                break;
            }

            assert!(start.elapsed().unwrap().as_secs() < timeout_secs, "timeout");

            sleep(Duration::from_millis(interval_millis)).await;
        }
    }

    pub async fn snapshot(&mut self) {
        self.snapshot = Some(self.introspect().await);
    }

    pub async fn poll_endpoint_for_changes(&mut self, timeout_secs: u64, interval_millis: u64) {
        let start = SystemTime::now();

        loop {
            // panic if a snapshot was not taken
            let snapshot = self.snapshot.clone().unwrap();

            match self.safe_introspect().await {
                Some(current) => {
                    if snapshot != current {
                        self.snapshot = Some(current);
                        break;
                    }
                }
                None => continue,
            };

            assert!(start.elapsed().unwrap().as_secs() < timeout_secs, "timeout");
            sleep(Duration::from_millis(interval_millis)).await;
        }
    }

    pub async fn get_playground_html(&self) -> String {
        self.client
            .get(&self.playground_endpoint)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap()
    }
}

#[derive(serde::Serialize)]
#[must_use]
pub struct GqlRequestBuilder<Response> {
    // These two will be serialized into the request
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<serde_json::Value>,

    // These won't
    #[serde(skip)]
    phantom: PhantomData<fn() -> Response>,
    #[serde(skip)]
    reqwest_builder: reqwest::RequestBuilder,
}

impl<Response> GqlRequestBuilder<Response> {
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.reqwest_builder = self.reqwest_builder.header(name.into(), value.into());
        self
    }

    pub fn variables(mut self, variables: impl serde::Serialize) -> Self {
        self.variables = Some(serde_json::to_value(variables).expect("to be able to serialize variables"));
        self
    }

    pub fn into_reqwest_builder(self) -> reqwest::RequestBuilder {
        let json = serde_json::to_value(&self).expect("to be able to serialize gql request");

        self.reqwest_builder.json(&json)
    }
}

impl<Response> IntoFuture for GqlRequestBuilder<Response>
where
    Response: serde::de::DeserializeOwned + 'static,
{
    type Output = Response;

    type IntoFuture = Pin<Box<dyn Future<Output = Response> + Send + 'static>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            self.into_reqwest_builder()
                .send()
                .await
                .unwrap()
                .json::<Response>()
                .await
                .unwrap()
        })
    }
}
