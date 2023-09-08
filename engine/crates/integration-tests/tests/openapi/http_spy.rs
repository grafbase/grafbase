use serde_json::Value;
use wiremock::{MockGuard, MockServer};

#[async_trait::async_trait]
pub trait ReceivedBodiesExt {
    async fn received_json_bodies(&self) -> Vec<Value>;
}

#[async_trait::async_trait]
impl ReceivedBodiesExt for MockServer {
    async fn received_json_bodies(&self) -> Vec<Value> {
        self.received_requests()
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|request| request.body_json().expect("expected JSON body"))
            .collect()
    }
}

#[async_trait::async_trait]
impl ReceivedBodiesExt for MockGuard {
    async fn received_json_bodies(&self) -> Vec<Value> {
        self.received_requests()
            .await
            .into_iter()
            .map(|request| request.body_json().expect("expected JSON body"))
            .collect()
    }
}
