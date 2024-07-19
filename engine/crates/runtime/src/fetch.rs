use std::{sync::Arc, time::Duration};

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
    pub url: &'a url::Url,
    pub headers: http::HeaderMap,
    pub json_body: String,
    pub subgraph_name: &'a str,
    pub timeout: Duration,
    pub retry_budget: Option<&'a tower::retry::budget::Budget>,
}

#[derive(Clone)]
pub struct FetchResponse {
    pub bytes: Bytes,
}

pub struct GraphqlRequest<'a> {
    pub url: &'a url::Url,
    pub headers: http::HeaderMap,
    pub query: &'a str,
    pub variables: Value,
}

#[async_trait::async_trait]
pub trait FetcherInner: Send + Sync {
    async fn post(&self, request: &FetchRequest<'_>) -> FetchResult<FetchResponse>;

    async fn stream(
        &self,
        request: GraphqlRequest<'_>,
    ) -> FetchResult<BoxStream<'static, Result<serde_json::Value, FetchError>>>;
}

#[derive(Clone)]
pub struct Fetcher {
    inner: Arc<dyn FetcherInner>,
}

impl Fetcher {
    pub fn new(fetcher: impl FetcherInner + 'static) -> Fetcher {
        Fetcher {
            inner: Arc::new(fetcher),
        }
    }

    #[allow(unused_variables)]
    pub async fn post<SleepFut>(
        &self,
        request: &FetchRequest<'_>,
        sleep: impl Fn(Duration) -> SleepFut + Send + Sync,
    ) -> FetchResult<FetchResponse>
    where
        SleepFut: std::future::Future<Output = ()> + Send,
    {
        let mut result = self.inner.post(request).await;

        let Some(retry_budget) = request.retry_budget else {
            return result;
        };

        loop {
            match result {
                Ok(bytes) => {
                    retry_budget.deposit();
                    return Ok(bytes);
                }
                Err(err) => {
                    if retry_budget.withdraw().is_ok() {
                        result = self.inner.post(request).await;
                    } else {
                        return Err(err);
                    }
                }
            }
        }
    }
}

impl std::ops::Deref for Fetcher {
    type Target = dyn FetcherInner;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}
