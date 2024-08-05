use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use futures::stream::BoxStream;
use graphql_mocks::ReceivedRequest;
use runtime::fetch::{FetchError, FetchRequest, FetchResponse, FetchResult, GraphqlRequest};

#[derive(Clone, Default)]
pub struct MockFetch {
    responses: Arc<Mutex<HashMap<String, crossbeam_queue::SegQueue<Vec<u8>>>>>,
    requests: Arc<crossbeam_queue::SegQueue<(String, ReceivedRequest)>>,
}

impl MockFetch {
    #[must_use]
    pub fn with_responses<R: serde::Serialize>(self, host: &str, responses: impl IntoIterator<Item = R>) -> Self {
        let mut responses_by_host = self.responses.lock().unwrap();
        let queue = responses_by_host.entry(host.to_string()).or_default();
        for response in responses {
            queue.push(serde_json::to_vec(&response).unwrap());
        }
        drop(responses_by_host);
        self
    }

    pub fn drain_received_requests(&self) -> impl Iterator<Item = (String, ReceivedRequest)> + '_ {
        std::iter::from_fn(|| self.requests.pop())
    }
}

#[async_trait::async_trait]
impl runtime::fetch::FetcherInner for MockFetch {
    async fn post(&self, request: &FetchRequest<'_>) -> FetchResult<FetchResponse> {
        let host = request.url.host_str().unwrap();
        self.requests.push((
            host.to_string(),
            ReceivedRequest {
                headers: request.headers.clone(),
                body: serde_json::from_slice(&request.json_body).unwrap(),
            },
        ));

        self.responses
            .lock()
            .unwrap()
            .get(host)
            .and_then(|responses| responses.pop())
            .map(|bytes| FetchResponse { bytes: bytes.into() })
            .ok_or(FetchError::any("No more responses"))
    }

    async fn stream(
        &self,
        _request: GraphqlRequest<'_>,
    ) -> FetchResult<BoxStream<'static, Result<serde_json::Value, FetchError>>> {
        unreachable!()
    }
}
