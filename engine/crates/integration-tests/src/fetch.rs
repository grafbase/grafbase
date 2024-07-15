use std::{collections::HashMap, sync::Arc};

use futures::stream::BoxStream;
use runtime::fetch::{FetchError, FetchRequest, FetchResponse, FetchResult, GraphqlRequest};
use url::Url;

#[derive(Default)]
pub struct MockFetch {
    responses: HashMap<String, crossbeam_queue::SegQueue<Vec<u8>>>,
}

impl MockFetch {
    #[must_use]
    pub fn with_responses<R: serde::Serialize>(mut self, host: &str, responses: impl IntoIterator<Item = R>) -> Self {
        let queue = self.responses.entry(host.to_string()).or_default();
        for response in responses {
            queue.push(serde_json::to_vec(&response).unwrap());
        }
        self
    }
}

#[async_trait::async_trait]
impl runtime::fetch::FetcherInner for MockFetch {
    async fn post(&self, request: FetchRequest<'_>) -> FetchResult<FetchResponse> {
        self.responses
            .get(request.url.host_str().unwrap())
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

pub type RecordedRequests = Arc<crossbeam_queue::SegQueue<RecordedSubRequest>>;

pub struct FetchRecorder {
    inner: runtime::fetch::Fetcher,
    url_to_subgraph_name: HashMap<Url, String>,
    requests: RecordedRequests,
}

impl FetchRecorder {
    pub fn record(inner: runtime::fetch::Fetcher) -> Self {
        Self {
            inner,
            url_to_subgraph_name: Default::default(),
            requests: Default::default(),
        }
    }

    #[must_use]
    pub fn with_url_to_subgraph_name(self, url_to_subgraph_name: HashMap<Url, String>) -> Self {
        Self {
            url_to_subgraph_name,
            ..self
        }
    }

    pub(crate) fn recorded_requests(&self) -> RecordedRequests {
        self.requests.clone()
    }
}

#[derive(serde::Serialize)]
pub struct RecordedSubRequest {
    pub subgraph_name: String,
    #[serde(skip)] // if you want them, serialize them somehow
    pub request_headers: http::HeaderMap,
    pub request_body: serde_json::Value,
    pub response_body: serde_json::Value,
}

#[async_trait::async_trait]
impl runtime::fetch::FetcherInner for FetchRecorder {
    async fn post(&self, request: FetchRequest<'_>) -> FetchResult<FetchResponse> {
        let mut record = RecordedSubRequest {
            subgraph_name: self.url_to_subgraph_name.get(request.url).cloned().unwrap_or_default(),
            request_body: serde_json::from_str(&request.json_body).unwrap_or_default(),
            request_headers: request.headers.clone(),
            response_body: Default::default(),
        };
        let response = self.inner.post(request).await?;
        record.response_body = serde_json::from_slice(&response.bytes).unwrap_or_default();
        self.requests.push(record);
        Ok(response)
    }

    async fn stream(
        &self,
        request: GraphqlRequest<'_>,
    ) -> FetchResult<BoxStream<'static, Result<serde_json::Value, FetchError>>> {
        self.inner.stream(request).await
    }
}
