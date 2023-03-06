#![allow(dead_code)]
use serde_json::json;
use std::time::{Duration, SystemTime};
use tokio::time::sleep;

use crate::utils::consts::INTROSPECTION_QUERY;

pub struct AsyncClient {
    endpoint: String,
    client: reqwest::Client,
    snapshot: Option<String>,
}

impl AsyncClient {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            client: reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(1))
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap(),
            snapshot: None,
        }
    }

    pub async fn gql<T>(&self, body: String) -> T
    where
        T: for<'de> serde::de::Deserialize<'de>,
    {
        self.client
            .post(&self.endpoint)
            .body(body)
            .send()
            .await
            .unwrap()
            .json::<T>()
            .await
            .unwrap()
    }

    async fn introspect(&self) -> String {
        self.client
            .post(&self.endpoint)
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
            .body(json!({"operationName":"IntrospectionQuery", "query": INTROSPECTION_QUERY}).to_string())
            .send()
            .await
        {
            if let Ok(text) = response.text().await {
                return Some(text);
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
            if self.client.head(&self.endpoint).send().await.is_ok() {
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
}
