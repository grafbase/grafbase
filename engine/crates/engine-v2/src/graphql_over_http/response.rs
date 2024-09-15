use bytes::Bytes;
use futures::{StreamExt, TryStreamExt};
use futures_util::Stream;
use grafbase_telemetry::graphql::GraphqlExecutionTelemetry;
use headers::HeaderMapExt;
use runtime::bytes::OwnedOrSharedBytes;

use crate::{
    engine::StreamResponse,
    response::{ErrorCode, ErrorCodeCounter, Response},
};

use super::{
    Body, CompleteResponseFormat, HooksExtension, ResponseFormat, StreamingResponseFormat, TelemetryExtension,
};

const APPLICATION_JSON: http::HeaderValue = http::HeaderValue::from_static("application/json");
const APPLICATION_GRAPHQL_RESPONSE_JSON: http::HeaderValue =
    http::HeaderValue::from_static("application/graphql-response+json");

pub(crate) struct Http;

impl Http {
    pub(crate) fn error(format: ResponseFormat, response: Response) -> http::Response<Body> {
        match format {
            ResponseFormat::Complete(format) => Self::from_complete_response_with_telemetry(format, &response),
            ResponseFormat::Streaming(format) => {
                let telemetry = TelemetryExtension::Ready(response.execution_telemetry());
                let mut http_response = Self::stream_from_first_response_and_rest_without_extensions(
                    format,
                    response,
                    futures_util::stream::empty(),
                );
                http_response.extensions_mut().insert(telemetry);
                http_response
            }
        }
    }

    pub(crate) fn single<C: Send + Sync + 'static>(
        format: CompleteResponseFormat,
        hooks_context: C,
        mut response: Response,
    ) -> http::Response<Body> {
        let mut http_response = Self::from_complete_response_with_telemetry(format, &response);
        http_response.extensions_mut().insert(HooksExtension::Single {
            context: hooks_context,
            on_operation_response_output: response.take_on_operation_response_output(),
        });

        http_response
    }

    pub(crate) fn batch<C: Send + Sync + 'static>(
        format: CompleteResponseFormat,
        hooks_context: C,
        mut responses: Vec<Response>,
    ) -> http::Response<Body> {
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

        let telemetry = {
            let counter = responses
                .iter()
                .fold(ErrorCodeCounter::default(), |mut counter, response| {
                    counter.add(response.error_code_counter());
                    counter
                });
            TelemetryExtension::Ready(GraphqlExecutionTelemetry {
                errors_count_by_code: counter.to_vec(),
                operations: responses
                    .iter()
                    .filter_map(|response| response.operation_attributes())
                    .map(|attributes| (attributes.ty, attributes.name.clone()))
                    .collect(),
            })
        };

        let hooks = HooksExtension::Batch {
            context: hooks_context,
            on_operation_response_outputs: responses
                .iter_mut()
                .filter_map(|response| response.take_on_operation_response_output())
                .collect(),
        };

        let mut headers = http::HeaderMap::new();
        headers.insert(
            http::header::CONTENT_TYPE,
            match format {
                CompleteResponseFormat::Json => APPLICATION_JSON,
                CompleteResponseFormat::GraphqlResponseJson => APPLICATION_GRAPHQL_RESPONSE_JSON,
            },
        );
        headers.typed_insert(headers::ContentLength(bytes.len() as u64));

        let mut http_response = http::Response::new(Body::Bytes(bytes));
        *http_response.status_mut() = status_code;
        *http_response.headers_mut() = headers;
        http_response.extensions_mut().insert(telemetry);
        http_response.extensions_mut().insert(hooks);

        http_response
    }

    pub(crate) async fn stream<C: Send + Sync + 'static>(
        format: StreamingResponseFormat,
        hooks_context: C,
        stream: StreamResponse,
    ) -> http::Response<Body> {
        let StreamResponse {
            mut stream,
            telemetry,
            on_operation_response_outputs,
        } = stream;
        let Some(first_response) = stream.next().await else {
            tracing::error!("Empty stream");
            return internal_server_error();
        };

        let mut http_response =
            Self::stream_from_first_response_and_rest_without_extensions(format, first_response, stream);
        http_response
            .extensions_mut()
            .insert(TelemetryExtension::Future(telemetry));
        http_response.extensions_mut().insert(HooksExtension::Stream {
            context: hooks_context,
            on_operation_response_outputs,
        });
        http_response
    }

    fn stream_from_first_response_and_rest_without_extensions(
        format: StreamingResponseFormat,
        response: Response,
        rest: impl Stream<Item = Response> + 'static + Send,
    ) -> http::Response<Body> {
        let status = compute_status_code(ResponseFormat::Streaming(format), &response);

        let (headers, stream) = gateway_core::encode_stream_response(
            futures_util::stream::iter(std::iter::once(response)).chain(rest),
            match format {
                StreamingResponseFormat::IncrementalDelivery => gateway_core::StreamingFormat::IncrementalDelivery,
                StreamingResponseFormat::GraphQLOverSSE => gateway_core::StreamingFormat::GraphQLOverSSE,
                StreamingResponseFormat::GraphQLOverWebSocket => {
                    unreachable!("Websocket response isn't returned as a HTTP response.")
                }
            },
        );

        let body = Body::Stream(stream.map_ok(|bytes| bytes.into()).boxed());
        let mut http_response = http::Response::new(body);
        *http_response.status_mut() = status;
        *http_response.headers_mut() = headers;

        http_response
    }

    fn from_complete_response_with_telemetry(
        format: CompleteResponseFormat,
        response: &Response,
    ) -> http::Response<Body> {
        let telemetry = TelemetryExtension::Ready(response.execution_telemetry());
        let bytes = match serde_json::to_vec(response) {
            Ok(bytes) => OwnedOrSharedBytes::Owned(bytes),
            Err(err) => {
                tracing::error!("Failed to serialize response: {err}");
                return internal_server_error();
            }
        };

        let status_code = compute_status_code(ResponseFormat::Complete(format), response);
        let mut headers = http::HeaderMap::new();

        headers.insert(http::header::CONTENT_TYPE, format.to_content_type());
        headers.typed_insert(headers::ContentLength(bytes.len() as u64));

        let mut http_response = http::Response::new(Body::Bytes(bytes));
        *http_response.status_mut() = status_code;
        *http_response.headers_mut() = headers;
        http_response.extensions_mut().insert(telemetry);
        http_response
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
                    .error_code_counter()
                    .iter()
                    .map(|(code, _)| code.into_http_status_code_with_priority())
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
        .extensions_mut()
        .insert(TelemetryExtension::Ready(GraphqlExecutionTelemetry {
            errors_count_by_code: vec![(ErrorCode::InternalServerError, 1)],
            operations: Vec::new(),
        }));

    response
}
