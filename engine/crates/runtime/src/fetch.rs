use std::sync::Arc;

use bytes::Bytes;
use futures_util::stream::BoxStream;
use serde_json::Value;

#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("{0}")]
    AnyError(String),
}

impl FetchError {
    pub fn any(error: impl ToString) -> Self {
        FetchError::AnyError(error.to_string())
    }
}

pub type FetchResult<T> = Result<T, FetchError>;

// very minimal for now, but will be expanded as we need it.
pub struct FetchRequest<'a> {
    pub url: &'a str,
    pub headers: Vec<(&'a str, &'a str)>,
    pub json_body: String,
}

#[derive(Clone)]
pub struct FetchResponse {
    pub bytes: Bytes,
}

pub struct GraphqlRequest<'a> {
    pub url: &'a str,
    pub headers: Vec<(&'a str, &'a str)>,
    pub query: String,
    pub variables: Value,
}

#[async_trait::async_trait]
pub trait FetcherInner: Send + Sync {
    async fn post(&self, request: FetchRequest<'_>) -> FetchResult<FetchResponse>;

    async fn stream(
        &self,
        request: GraphqlRequest<'_>,
    ) -> FetchResult<BoxStream<'static, Result<serde_json::Value, FetchError>>>;
}

pub struct Fetcher {
    inner: Arc<dyn FetcherInner>,
}

impl Fetcher {
    pub fn new(fetcher: impl FetcherInner + 'static) -> Fetcher {
        Fetcher {
            inner: Arc::new(fetcher),
        }
    }
}

impl std::ops::Deref for Fetcher {
    type Target = dyn FetcherInner;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}
