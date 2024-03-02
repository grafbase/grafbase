mod metadata;
mod streaming;

use futures_util::{stream::BoxStream, Stream};
use headers::HeaderMapExt;
use itertools::Itertools;
pub use metadata::*;
use runtime::{bytes::OwnedOrSharedBytes, cache::CachedResponse};
pub use streaming::*;

pub struct HttpGraphqlResponse {
    pub headers: http::HeaderMap,
    pub body: ResponseBody,
    // TODO: remove me when tail workers will be used for analytics.
    pub metadata: ExecutionMetadata,
}

pub enum ResponseBody {
    Bytes(OwnedOrSharedBytes),
    Stream(BoxStream<'static, Result<OwnedOrSharedBytes, String>>),
}

impl HttpGraphqlResponse {
    pub fn with_metadata(mut self, metadata: ExecutionMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn from_bytes(bytes: OwnedOrSharedBytes) -> HttpGraphqlResponse {
        let mut headers = http::HeaderMap::new();
        headers.typed_insert(headers::ContentLength(bytes.len() as u64));
        HttpGraphqlResponse {
            headers,
            body: ResponseBody::Bytes(bytes),
            metadata: ExecutionMetadata::default(),
        }
    }

    pub fn from_json_bytes(bytes: OwnedOrSharedBytes) -> HttpGraphqlResponse {
        let mut response = Self::from_bytes(bytes);
        response.headers.typed_insert(headers::ContentType::json());
        response
    }

    pub fn from_json(value: &impl serde::Serialize) -> HttpGraphqlResponse {
        match serde_json::to_vec(value) {
            Ok(bytes) => Self::from_json_bytes(bytes.into()),
            Err(err) => {
                tracing::error!("Failed to serialize response: {}", err);
                Self::error("Internal Server Error")
            }
        }
    }

    pub fn unauthorized() -> HttpGraphqlResponse {
        Self::error("Unauthorized")
    }

    pub fn error(message: &str) -> HttpGraphqlResponse {
        Self::from_json_bytes(
            serde_json::to_vec(&serde_json::json!({
                "errors": [
                    {
                        "message": message,
                    }
                ]
            }))
            .expect("valid json")
            .into(),
        )
    }

    pub async fn batch_response(responses: Vec<HttpGraphqlResponse>) -> HttpGraphqlResponse {
        let mut bytes_batch = Vec::new();
        for response in responses {
            // Sanity check
            assert_eq!(
                response.headers.typed_get::<headers::ContentType>(),
                Some(headers::ContentType::json())
            );
            let ResponseBody::Bytes(bytes) = response.body else {
                return Self::error("Cannot use stream response with batch request.");
            };
            bytes_batch.push(bytes);
        }
        let mut body = Vec::with_capacity(
            // '[]' + commas + actual bodies
            2 + (bytes_batch.len() - 1) + bytes_batch.iter().map(|bytes| bytes.len()).sum::<usize>(),
        );
        body.push(b'[');
        for bytes in Itertools::intersperse(bytes_batch.iter().map(|bytes| bytes.as_ref()), &[b',']) {
            body.extend_from_slice(bytes);
        }
        body.push(b']');
        HttpGraphqlResponse::from_json_bytes(body.into())
    }

    pub async fn from_stream<T>(
        ray_id: &str,
        format: StreamingFormat,
        stream: impl Stream<Item = T> + Send + 'static,
    ) -> Self
    where
        T: serde::Serialize + Send,
    {
        let (headers, stream) = self::streaming::encode_stream_response(ray_id.to_string(), stream, format).await;
        Self {
            headers,
            body: ResponseBody::Stream(stream),
            metadata: ExecutionMetadata::default(),
        }
    }
}

impl From<CachedResponse> for HttpGraphqlResponse {
    fn from(cached: CachedResponse) -> Self {
        let mut response = Self::from_json_bytes(cached.body);
        response.headers.typed_insert(cached.status);
        response.headers.typed_insert(cached.cache_control);
        response
    }
}

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for HttpGraphqlResponse {
    fn into_response(self) -> axum::response::Response {
        let HttpGraphqlResponse { headers, body, .. } = self;
        match body {
            ResponseBody::Bytes(bytes) => match bytes {
                OwnedOrSharedBytes::Owned(bytes) => (headers, bytes).into_response(),
                OwnedOrSharedBytes::Shared(bytes) => (headers, bytes).into_response(),
            },
            ResponseBody::Stream(stream) => (headers, axum::body::Body::from_stream(stream)).into_response(),
        }
    }
}
