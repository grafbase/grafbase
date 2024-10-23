use std::{borrow::Cow, future::Future, time::Duration};

use bytes::Bytes;
use futures_util::{stream::BoxStream, Stream, StreamExt, TryFutureExt};

use crate::{bytes::OwnedOrSharedBytes, hooks::ResponseInfo};

#[derive(Clone, Debug, thiserror::Error)]
pub enum FetchError {
    #[error("{0}")]
    AnyError(String),
    #[error("Timeout")]
    Timeout,
    #[error("Invalid status code: {0:?}")]
    InvalidStatusCode(http::StatusCode),
}

impl FetchError {
    pub fn any(error: impl ToString) -> Self {
        FetchError::AnyError(error.to_string())
    }

    pub fn as_invalid_status_code(&self) -> Option<http::StatusCode> {
        match self {
            FetchError::InvalidStatusCode(status) => Some(*status),
            _ => None,
        }
    }
}

pub type FetchResult<T> = Result<T, FetchError>;

/// reqwest uses Url instead of Uri, so as long as it's the actual implementation underneath it's a
/// bit of a waste to use http::Request
#[derive(Clone)]
pub struct FetchRequest<'a, Body> {
    pub url: Cow<'a, url::Url>,
    pub method: http::Method,
    pub headers: http::HeaderMap,
    pub body: Body,
    pub timeout: Duration,
}

pub trait Fetcher: Send + Sync + 'static {
    fn fetch(
        &self,
        request: FetchRequest<'_, Bytes>,
    ) -> impl Future<Output = (FetchResult<http::Response<OwnedOrSharedBytes>>, Option<ResponseInfo>)> + Send;

    fn graphql_over_sse_stream(
        &self,
        request: FetchRequest<'_, Bytes>,
    ) -> impl Future<Output = FetchResult<impl Stream<Item = FetchResult<OwnedOrSharedBytes>> + Send + 'static>> + Send;

    // graphql_ws_client requires a serde::Serialize
    fn graphql_over_websocket_stream<T>(
        &self,
        request: FetchRequest<'_, T>,
    ) -> impl Future<Output = FetchResult<impl Stream<Item = FetchResult<serde_json::Value>> + Send + 'static>> + Send
    where
        T: serde::Serialize + Send;
}

pub mod dynamic {
    use super::*;

    #[allow(unused_variables)] // makes it easier to copy-paste relevant functions
    #[async_trait::async_trait]
    pub trait DynFetcher: Send + Sync + 'static {
        async fn fetch(
            &self,
            request: FetchRequest<'_, Bytes>,
        ) -> (FetchResult<http::Response<OwnedOrSharedBytes>>, Option<ResponseInfo>);

        async fn graphql_over_sse_stream(
            &self,
            request: FetchRequest<'_, Bytes>,
        ) -> FetchResult<BoxStream<'static, FetchResult<OwnedOrSharedBytes>>> {
            unreachable!()
        }

        async fn graphql_over_websocket_stream(
            &self,
            request: FetchRequest<'_, serde_json::Value>,
        ) -> FetchResult<BoxStream<'static, FetchResult<serde_json::Value>>> {
            unreachable!()
        }
    }

    pub struct DynamicFetcher(Box<dyn DynFetcher>);

    impl<T: DynFetcher> From<T> for DynamicFetcher {
        fn from(fetcher: T) -> Self {
            Self::new(fetcher)
        }
    }

    impl DynamicFetcher {
        pub fn wrap(fetcher: impl Fetcher) -> Self {
            Self::new(DynWrapper(fetcher))
        }

        pub fn new(fetcher: impl DynFetcher) -> Self {
            Self(Box::new(fetcher))
        }
    }

    impl Fetcher for DynamicFetcher {
        async fn fetch(
            &self,
            request: FetchRequest<'_, Bytes>,
        ) -> (FetchResult<http::Response<OwnedOrSharedBytes>>, Option<ResponseInfo>) {
            self.0.fetch(request).await
        }

        async fn graphql_over_sse_stream(
            &self,
            request: FetchRequest<'_, Bytes>,
        ) -> FetchResult<impl Stream<Item = FetchResult<OwnedOrSharedBytes>> + Send + 'static> {
            self.0.graphql_over_sse_stream(request).await
        }

        async fn graphql_over_websocket_stream<T>(
            &self,
            request: FetchRequest<'_, T>,
        ) -> FetchResult<impl Stream<Item = FetchResult<serde_json::Value>> + Send + 'static>
        where
            T: serde::Serialize + Send,
        {
            self.0
                .graphql_over_websocket_stream(FetchRequest {
                    method: request.method,
                    url: request.url,
                    headers: request.headers,
                    body: serde_json::to_value(request.body).unwrap(),
                    timeout: request.timeout,
                })
                .await
        }
    }

    struct DynWrapper<T>(T);

    #[async_trait::async_trait]
    impl<T: Fetcher> DynFetcher for DynWrapper<T> {
        async fn fetch(
            &self,
            request: FetchRequest<'_, Bytes>,
        ) -> (FetchResult<http::Response<OwnedOrSharedBytes>>, Option<ResponseInfo>) {
            self.0.fetch(request).await
        }

        async fn graphql_over_sse_stream(
            &self,
            request: FetchRequest<'_, Bytes>,
        ) -> FetchResult<BoxStream<'static, FetchResult<OwnedOrSharedBytes>>> {
            self.0
                .graphql_over_sse_stream(request)
                .map_ok(|stream| stream.boxed())
                .await
        }

        async fn graphql_over_websocket_stream(
            &self,
            request: FetchRequest<'_, serde_json::Value>,
        ) -> FetchResult<BoxStream<'static, FetchResult<serde_json::Value>>> {
            self.0
                .graphql_over_websocket_stream(request)
                .map_ok(|stream| stream.boxed())
                .await
        }
    }
}
