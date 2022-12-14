#![allow(dead_code)]
use reqwest::header::HeaderMap;
use serde_json::json;
use std::{
    thread::sleep,
    time::{Duration, SystemTime},
};

use crate::utils::consts::INTROSPECTION_QUERY;

pub struct Client {
    endpoint: String,
    headers: HeaderMap,
    client: reqwest::blocking::Client,
    snapshot: Option<String>,
}

impl Client {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            headers: HeaderMap::new(),
            client: reqwest::blocking::Client::new(),
            snapshot: None,
        }
    }

    pub fn with_header(mut self, key: &'static str, value: String) -> Self {
        self.headers.insert(key, value.parse().unwrap());
        self
    }

    pub fn gql<T>(&self, body: String) -> T
    where
        T: for<'de> serde::de::Deserialize<'de>,
    {
        self.client
            .post(&self.endpoint)
            .body(body)
            .headers(self.headers.clone())
            .send()
            .unwrap()
            .json::<T>()
            .unwrap()
    }

    fn introspect(&self) -> String {
        self.client
            .post(&self.endpoint)
            .body(json!({"operationName":"IntrospectionQuery", "query": INTROSPECTION_QUERY}).to_string())
            .headers(self.headers.clone())
            .send()
            .unwrap()
            .text()
            .unwrap()
    }

    fn safe_introspect(&self) -> Option<String> {
        if let Ok(response) = self
            .client
            .post(&self.endpoint)
            .body(json!({"operationName":"IntrospectionQuery", "query": INTROSPECTION_QUERY}).to_string())
            .headers(self.headers.clone())
            .send()
        {
            if let Ok(text) = response.text() {
                return Some(text);
            }
        }

        None
    }

    /// # Panics
    ///
    /// panics if the set timeout is reached
    pub fn poll_endpoint(&self, timeout_secs: u64, interval_millis: u64) {
        let start = SystemTime::now();

        loop {
            if self.client.head(&self.endpoint).send().is_ok() {
                break;
            }

            assert!(start.elapsed().unwrap().as_secs() < timeout_secs, "timeout");

            sleep(Duration::from_millis(interval_millis));
        }
    }

    pub fn snapshot(&mut self) {
        self.snapshot = Some(self.introspect());
    }

    pub fn poll_endpoint_for_changes(&mut self, timeout_secs: u64, interval_millis: u64) {
        let start = SystemTime::now();

        loop {
            // panic if a snapshot was not taken
            let snapshot = self.snapshot.clone().unwrap();

            match self.safe_introspect() {
                Some(current) => {
                    if snapshot != current {
                        self.snapshot = Some(current);
                        break;
                    }
                }
                None => continue,
            };

            assert!(start.elapsed().unwrap().as_secs() < timeout_secs, "timeout");
            sleep(Duration::from_millis(interval_millis));
        }
    }
}
