use bytes::Bytes;
use enumset::EnumSet;
use futures::{StreamExt, TryStreamExt};
use futures_util::Stream;
use grafbase_telemetry::gql_response_status::GraphqlResponseStatus;
use headers::HeaderMapExt;
use runtime::bytes::OwnedOrSharedBytes;

use crate::response::{ErrorCode, Response};

use super::{Body, CompleteResponseFormat, ResponseFormat, StreamingResponseFormat};

const APPLICATION_JSON: http::HeaderValue = http::HeaderValue::from_static("application/json");
const APPLICATION_GRAPHQL_RESPONSE_JSON: http::HeaderValue =
    http::HeaderValue::from_static("application/graphql-response+json");

pub(crate) struct Http;

impl Http {
    pub(crate) fn from(format: ResponseFormat, response: Response) -> http::Response<Body> {
        match format {
            ResponseFormat::Complete(format) => Self::single(format, response),
            ResponseFormat::Streaming(format) => {
                Self::stream_from_first_response_and_rest(format, response, futures_util::stream::empty())
            }
        }
    }

    pub(crate) fn single(format: CompleteResponseFormat, response: Response) -> http::Response<Body> {
        let bytes = match serde_json::to_vec(&response) {
            Ok(bytes) => OwnedOrSharedBytes::Owned(bytes),
            Err(err) => {
                tracing::error!("Failed to serialize response: {err}");
                return internal_server_error();
            }
        };

        let status_code = compute_status_code(ResponseFormat::Complete(format), &response);
        let mut headers = http::HeaderMap::new();

        headers.typed_insert(response.graphql_status());
        headers.insert(http::header::CONTENT_TYPE, format.to_content_type());
        headers.typed_insert(headers::ContentLength(bytes.len() as u64));

        let mut response = http::Response::new(Body::Bytes(bytes));
        *response.status_mut() = status_code;
        *response.headers_mut() = headers;

        response
    }

    pub(crate) fn batch(format: CompleteResponseFormat, responses: Vec<Response>) -> http::Response<Body> {
        let bytes = match serde_json::to_vec(&responses) {
            Ok(bytes) => OwnedOrSharedBytes::Owned(bytes),
            Err(err) => {
                tracing::error!("Failed to serialize response: {err}");
                return internal_server_error();
            }
        };

        let status_code = responses.iter().fold(http::StatusCode::OK, |status, response| {
            if !status.is_client_error() {
                let other = compute_status_code(ResponseFormat::Complete(format), response);
                if !other.is_success() {
                    return other;
                }
            }
            status
        });

        let graphql_status = responses
            .iter()
            .fold(GraphqlResponseStatus::Success, |graphql_status, response| {
                graphql_status.union(response.graphql_status())
            });

        let mut headers = http::HeaderMap::new();
        headers.typed_insert(graphql_status);

        headers.insert(
            http::header::CONTENT_TYPE,
            match format {
                CompleteResponseFormat::Json => APPLICATION_JSON,
                CompleteResponseFormat::GraphqlResponseJson => APPLICATION_GRAPHQL_RESPONSE_JSON,
            },
        );

        headers.typed_insert(headers::ContentLength(bytes.len() as u64));

        let mut response = http::Response::new(Body::Bytes(bytes));
        *response.status_mut() = status_code;
        *response.headers_mut() = headers;

        response
    }

    pub(crate) async fn stream(
        format: StreamingResponseFormat,
        stream: impl Stream<Item = Response> + 'static + Send,
    ) -> http::Response<Body> {
        let mut stream = Box::pin(stream);
        let Some(first_response) = stream.next().await else {
            tracing::error!("Empty stream");
            return internal_server_error();
        };

        Self::stream_from_first_response_and_rest(format, first_response, stream)
    }

    fn stream_from_first_response_and_rest(
        format: StreamingResponseFormat,
        first: Response,
        rest: impl Stream<Item = Response> + 'static + Send,
    ) -> http::Response<Body> {
        let graphql_status = first.graphql_status();
        let status = compute_status_code(ResponseFormat::Streaming(format), &first);

        let (mut headers, stream) = gateway_core::encode_stream_response(
            futures_util::stream::iter(std::iter::once(first)).chain(rest),
            match format {
                StreamingResponseFormat::IncrementalDelivery => gateway_core::StreamingFormat::IncrementalDelivery,
                StreamingResponseFormat::GraphQLOverSSE => gateway_core::StreamingFormat::GraphQLOverSSE,
                StreamingResponseFormat::GraphQLOverWebSocket => {
                    unreachable!("Websocket response isn't returned as a HTTP response.")
                }
            },
        );
        headers.typed_insert(graphql_status);

        let mut response = http::Response::new(Body::Stream(stream.map_ok(|bytes| bytes.into()).boxed()));
        *response.status_mut() = status;
        *response.headers_mut() = headers;

        response
    }
}

fn compute_status_code(format: ResponseFormat, response: &Response) -> http::StatusCode {
    match response {
        // GraphQL-over-HTTP spec:
        //   A server MAY forbid individual requests by a client to any endpoint for any reason, for example
        //   to require authentication or payment; when doing so it SHOULD use the relevant 4xx or 5xx status code.
        //   This decision SHOULD NOT be based on the contents of a well-formed GraphQL-over-HTTP request.
        //
        //   In case of errors that completely prevent the generation of a well-formed GraphQL response,
        //   the server SHOULD respond with the appropriate status code depending on the concrete error condition,
        //   and MUST NOT respond with a 2xx status code when using the application/graphql-response+json media type.
        Response::RefusedRequest(resp) => resp.status(),
        Response::RequestError(_) => {
            match format {
                // GraphQL-over-HTTP spec:
                //   The server SHOULD use the 200 status code for every response to a well-formed GraphQL-over-HTTP request,
                //   independent of any GraphQL request error or GraphQL field error raised.
                //   If the GraphQL response contains a non-null {data} entry then the server MUST use the 200 status code.
                //
                // So we're always returning 200. Hooks may return a non-200 status code as long we didn't
                // start execution or data is not present.
                ResponseFormat::Complete(CompleteResponseFormat::Json) => http::StatusCode::OK,
                // GraphQL over SSE spec:
                //   Validation steps that run before execution of the GraphQL operation MUST report errors through an accepted SSE connection
                //   by emitting next events that contain the errors in the data field. One reason being, the server should agree with the client's
                //   Accept header when deciding about the response's Content-Type. Additionally, responding with a 400 (Bad Request) will cause the
                //   user agent to fail the connection. In some cases, like with the browser's native EventSource, the error event will hold no
                //   meaningful information helping to understand the validation issue(s).
                ResponseFormat::Streaming(StreamingResponseFormat::GraphQLOverSSE) => http::StatusCode::OK,
                // GraphQL-over-HTTP spec:
                //   If the GraphQL response does not contain the {data} entry then
                //   the server MUST reply with a 4xx or 5xx status code as appropriate.
                ResponseFormat::Complete(CompleteResponseFormat::GraphqlResponseJson) => response
                    .errors()
                    .iter()
                    .fold(EnumSet::<ErrorCode>::empty(), |mut set, error| {
                        set |= error.code;
                        set
                    })
                    .into_iter()
                    .map(ErrorCode::into_http_status_code_with_priority)
                    .max_by_key(|(_, priority)| *priority)
                    .map(|(status, _)| status)
                    .unwrap_or(http::StatusCode::INTERNAL_SERVER_ERROR),
                // Unclear what we should do here. We should probably check for the presence of
                // 'application/graphql-response+json' in the Accept header. In the meantime, we'll
                // assume it's `"application/json"`.
                ResponseFormat::Streaming(StreamingResponseFormat::IncrementalDelivery) => http::StatusCode::OK,
                ResponseFormat::Streaming(StreamingResponseFormat::GraphQLOverWebSocket) => {
                    unreachable!("HTTP status code has no meaning in a websocket connection")
                }
            }
        }
        Response::Executed(_) => {
            // GraphQL-over-HTTP spec:
            //   If the GraphQL response contains the {data} entry and it is {null}, then the server SHOULD
            //   reply with a 2xx status code and it is RECOMMENDED it replies with 200 status code.
            //
            //   If the GraphQL response contains the {data} entry and it is not {null}, then
            //   the server MUST reply with a 2xx status code and SHOULD reply with 200 status code.
            http::StatusCode::OK
        }
    }
}

fn internal_server_error() -> http::Response<Body> {
    let body = Bytes::from_static(
        br###"{"errors":[{"message":"Internal server error","extensions":{"code":"INTERNAL_SERVER_ERROR"}}]}"###,
    );
    let mut headers = http::HeaderMap::new();
    headers.insert(http::header::CONTENT_TYPE, APPLICATION_JSON.clone());
    headers.typed_insert(headers::ContentLength(body.len() as u64));
    let mut response = http::Response::new(Body::Bytes(OwnedOrSharedBytes::Shared(body)));
    *response.status_mut() = http::StatusCode::INTERNAL_SERVER_ERROR;
    *response.headers_mut() = headers;

    response
}
