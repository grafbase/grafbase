use futures::{StreamExt, TryStreamExt};
use futures_util::{stream::BoxStream, Stream};
use gateway_core::StreamingFormat;
use grafbase_telemetry::gql_response_status::GraphqlResponseStatus;
use headers::HeaderMapExt;
use runtime::bytes::OwnedOrSharedBytes;

use crate::response::{ErrorCode, Response};

/// A GraphQL response with HTTP headers and execution metadata (used for tracing).
/// The response is already pre-serialized because it might be coming directly from the cache.
pub struct HttpGraphqlResponse {
    pub headers: http::HeaderMap,
    pub body: HttpGraphqlResponseBody,
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
    pub fn bad_request_error(message: &str) -> HttpGraphqlResponse {
        Self::from_json(
            GraphqlResponseStatus::RequestError { count: 1 },
            &serde_json::json!({
            "errors": [
                {
                    "message": message,
                    "extensions": {
                        "code": ErrorCode::BadRequest
                    }
                }
            ]
            }),
        )
    }

    pub fn internal_server_error(message: &str) -> HttpGraphqlResponse {
        Self::from_json(
            GraphqlResponseStatus::RequestError { count: 1 },
            &serde_json::json!({
                "errors": [
                    {
                        "message": message,
                        "extensions": {
                            "code": ErrorCode::InternalServerError
                        }
                    }
                ]
            }),
        )
    }

    pub(crate) fn build(response: Response, format: Option<StreamingFormat>) -> Self {
        if let Some(format) = format {
            Self::from_stream(
                format,
                response.status(),
                futures_util::stream::iter(std::iter::once(response)),
            )
        } else {
            Self::from_json(response.status(), &response)
        }
    }

    pub(crate) fn from_stream<T>(
        format: StreamingFormat,
        status: GraphqlResponseStatus,
        stream: impl Stream<Item = T> + Send + 'static,
    ) -> Self
    where
        T: serde::Serialize + Send,
    {
        let (mut headers, stream) = gateway_core::encode_stream_response(stream, format);
        headers.typed_insert(status);
        Self {
            headers,
            body: HttpGraphqlResponseBody::Stream(stream.map_ok(|bytes| bytes.into()).boxed()),
        }
    }

    pub(crate) fn from_batch(responses: Vec<HttpGraphqlResponse>) -> HttpGraphqlResponse {
        // Currently we only output JSON and those can be easily stitched together for a batch
        // response so we avoid a serde round-trip.
        let mut bytes_batch = Vec::new();
        let mut status = GraphqlResponseStatus::Success;
        for response in responses {
            // Sanity check
            assert_eq!(
                response.headers.typed_get::<headers::ContentType>(),
                Some(headers::ContentType::json())
            );
            // Kind of best effort at this stage to return something sensible for the request
            // trace/metric
            if let Some(response_status) = response.headers.typed_get::<GraphqlResponseStatus>() {
                status = status.union(response_status);
            }
            let HttpGraphqlResponseBody::Bytes(bytes) = response.body else {
                tracing::error!("Cannot use stream response with batch request.");
                return Self::internal_server_error("Internal server error");
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
        HttpGraphqlResponse::from_json_bytes(status, body.into())
    }

    fn from_json(status: GraphqlResponseStatus, value: &impl serde::Serialize) -> HttpGraphqlResponse {
        match serde_json::to_vec(value) {
            Ok(bytes) => Self::from_json_bytes(status, bytes.into()),
            Err(err) => {
                tracing::error!("Failed to serialize response: {}", err);
                Self::internal_server_error("Internal server error")
            }
        }
    }

    fn from_json_bytes(status: GraphqlResponseStatus, bytes: OwnedOrSharedBytes) -> HttpGraphqlResponse {
        let mut response = Self::from_bytes(status, bytes);
        response.headers.typed_insert(headers::ContentType::json());
        response
    }

    fn from_bytes(status: GraphqlResponseStatus, bytes: OwnedOrSharedBytes) -> HttpGraphqlResponse {
        let mut headers = http::HeaderMap::new();
        headers.typed_insert(status);
        headers.typed_insert(headers::ContentLength(bytes.len() as u64));
        HttpGraphqlResponse {
            headers,
            body: HttpGraphqlResponseBody::Bytes(bytes),
        }
    }
}
