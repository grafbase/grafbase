use std::{
    thread::sleep,
    time::{Duration, SystemTime},
};

pub struct Client {
    endpoint: String,
    client: reqwest::blocking::Client,
}

impl Client {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            client: reqwest::blocking::Client::new(),
        }
    }
    pub fn gql<T>(&self, body: String) -> T
    where
        T: for<'de> serde::de::Deserialize<'de>,
    {
        self.client
            .post(&self.endpoint)
            .body(body)
            .send()
            .unwrap()
            .json::<T>()
            .unwrap()
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
}
