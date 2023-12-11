use std::sync::Arc;

use bytes::Bytes;

#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("{0}")]
    AnyError(String),
}

pub type FetchResult<T> = Result<T, FetchError>;

// very minimal for now, but will be expanded as we need it.
pub struct FetchRequest<'a> {
    pub url: &'a str,
    pub headers: Vec<(&'a str, &'a str)>,
    pub json_body: String,
}

pub struct FetchResponse {
    pub bytes: Bytes,
}

#[async_trait::async_trait]
pub trait FetcherInner {
    async fn post(&self, request: FetchRequest<'_>) -> FetchResult<FetchResponse>;
}

type BoxedFetcherImpl = Box<dyn FetcherInner + Send + Sync>;

pub struct Fetcher {
    inner: Arc<BoxedFetcherImpl>,
}

impl Fetcher {
    pub fn new(fetcher: BoxedFetcherImpl) -> Fetcher {
        Fetcher {
            inner: Arc::new(fetcher),
        }
    }
}

impl std::ops::Deref for Fetcher {
    type Target = BoxedFetcherImpl;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
