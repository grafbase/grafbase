use futures::{StreamExt, TryStreamExt};
use futures_util::{stream::BoxStream, Stream};
use gateway_core::StreamingFormat;
use headers::HeaderMapExt;
use runtime::bytes::OwnedOrSharedBytes;

/// A GraphQL response with HTTP headers and execution metadata (used for tracing).
/// The response is already pre-serialized because it might be coming directly from the cache.
pub struct HttpGraphqlResponse {
    pub headers: http::HeaderMap,
    pub body: HttpGraphqlResponseBody,
    // TODO: Used to propagate this metadata to headers for our current analytics on Cloudflare.
    //       It should not be relied upon otherwise, doesn't work well for batch requests and will
    //       be removed once we also use otel for the managed version.
    pub metadata: OperationMetadata,
}

#[derive(Default)]
pub struct OperationMetadata {
    pub operation_name: Option<String>,
    pub operation_type: Option<&'static str>,
    pub has_errors: bool,
}

pub enum HttpGraphqlResponseBody {
    Bytes(OwnedOrSharedBytes),
    Stream(BoxStream<'static, Result<OwnedOrSharedBytes, String>>),
}

impl HttpGraphqlResponseBody {
    pub fn into_stream(self) -> BoxStream<'static, Result<OwnedOrSharedBytes, String>> {
        match self {
            HttpGraphqlResponseBody::Bytes(bytes) => Box::pin(futures_util::stream::once(async move { Ok(bytes) })),
            HttpGraphqlResponseBody::Stream(stream) => stream,
        }
    }
}

impl HttpGraphqlResponse {
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

    pub(crate) fn with_metadata(mut self, metadata: OperationMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    pub(crate) fn from_bytes(bytes: OwnedOrSharedBytes) -> HttpGraphqlResponse {
        let mut headers = http::HeaderMap::new();
        headers.typed_insert(headers::ContentLength(bytes.len() as u64));
        HttpGraphqlResponse {
            headers,
            metadata: OperationMetadata::default(),
            body: HttpGraphqlResponseBody::Bytes(bytes),
        }
    }

    pub(crate) fn from_json_bytes(bytes: OwnedOrSharedBytes) -> HttpGraphqlResponse {
        let mut response = Self::from_bytes(bytes);
        response.headers.typed_insert(headers::ContentType::json());
        response
    }

    pub(crate) fn from_json(value: &impl serde::Serialize) -> HttpGraphqlResponse {
        match serde_json::to_vec(value) {
            Ok(bytes) => Self::from_json_bytes(bytes.into()),
            Err(err) => {
                tracing::error!("Failed to serialize response: {}", err);
                Self::error("Internal Server Error")
            }
        }
    }

    pub(crate) fn batch_response(responses: Vec<HttpGraphqlResponse>) -> HttpGraphqlResponse {
        // Currently we only output JSON and those can be easily stitched together for a batch
        // response so we avoid a serde round-trip.
        let mut bytes_batch = Vec::new();
        for response in responses {
            // Sanity check
            assert_eq!(
                response.headers.typed_get::<headers::ContentType>(),
                Some(headers::ContentType::json())
            );
            let HttpGraphqlResponseBody::Bytes(bytes) = response.body else {
                return Self::error("Cannot use stream response with batch request.");
            };
            bytes_batch.push(bytes);
        }
        let mut commas_count = bytes_batch.len() - 1;
        let mut body = Vec::with_capacity(
            // '[]' + commas + actual bodies
            2 + commas_count + bytes_batch.iter().map(|bytes| bytes.len()).sum::<usize>(),
        );
        body.push(b'[');
        for bytes in bytes_batch {
            body.extend_from_slice(bytes.as_ref());
            if commas_count > 0 {
                body.push(b',');
                commas_count -= 1;
            }
        }
        body.push(b']');
        HttpGraphqlResponse::from_json_bytes(body.into())
    }

    pub(crate) fn stream_error(format: StreamingFormat, message: &str) -> HttpGraphqlResponse {
        Self::from_stream(
            format,
            futures_util::stream::iter(std::iter::once(serde_json::json!({
                "errors": [
                    {
                        "message": message,
                    }
                ]
            }))),
        )
    }

    pub(crate) fn from_stream<T>(format: StreamingFormat, stream: impl Stream<Item = T> + Send + 'static) -> Self
    where
        T: serde::Serialize + Send,
    {
        let (headers, stream) = gateway_core::encode_stream_response(stream, format);
        Self {
            headers,
            metadata: OperationMetadata::default(),
            body: HttpGraphqlResponseBody::Stream(stream.map_ok(|bytes| bytes.into()).boxed()),
        }
    }
}
