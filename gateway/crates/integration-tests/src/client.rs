use std::time::{Duration, SystemTime};

use http::{HeaderMap, StatusCode};

use crate::{CommandHandles, GraphQlRequestBody, TestBatchRequest, TestRequest};

pub struct Client {
    endpoint: String,
    client: reqwest::Client,
    headers: HeaderMap,
    commands: CommandHandles,
}

impl Client {
    pub fn new(endpoint: String, commands: CommandHandles) -> Self {
        Self {
            endpoint,
            headers: HeaderMap::new(),
            client: reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(1))
                .build()
                .unwrap(),
            commands,
        }
    }

    pub fn with_header(mut self, key: &'static str, value: impl AsRef<str>) -> Self {
        self.headers.insert(key, value.as_ref().parse().unwrap());
        self
    }

    pub async fn poll_endpoint(&self, timeout_secs: u64, interval_millis: u64) {
        let start = SystemTime::now();

        loop {
            let valid_response = self
                .client
                .head(&self.endpoint)
                .send()
                .await
                .is_ok_and(|response| response.status() != StatusCode::SERVICE_UNAVAILABLE);

            if !self.commands.still_running() {
                panic!("commands are no longer running so polling is bound to fail");
            }

            if valid_response {
                break;
            }

            assert!(start.elapsed().unwrap().as_secs() < timeout_secs, "timeout");

            tokio::time::sleep(Duration::from_millis(interval_millis)).await;
        }
    }

    pub fn kill_handles(&self) {
        self.commands.kill_all()
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn execute(&self, body: impl Into<GraphQlRequestBody>) -> TestRequest {
        if !self.commands.still_running() {
            panic!("commands are no longer running so execute is bound to fail");
        }

        let (mut parts, _) = http::Request::new(()).into_parts();
        parts.method = http::Method::POST;
        parts.uri = self.endpoint.parse().unwrap();

        TestRequest {
            client: self.client.clone(),
            parts,
            body: body.into(),
        }
    }

    pub fn execute_batch<T: Into<GraphQlRequestBody>>(&self, bodies: impl IntoIterator<Item = T>) -> TestBatchRequest {
        if !self.commands.still_running() {
            panic!("commands are no longer running so execute is bound to fail");
        }

        let (mut parts, _) = http::Request::new(()).into_parts();
        parts.method = http::Method::POST;
        parts.uri = self.endpoint.parse().unwrap();

        TestBatchRequest {
            client: self.client.clone(),
            parts,
            body: bodies.into_iter().map(Into::into).collect(),
        }
    }

    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }
}
