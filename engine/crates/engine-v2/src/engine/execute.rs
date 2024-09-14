use ::runtime::{error::ErrorResponse, hooks::Hooks, rate_limiting::RateLimitKey};
use bytes::Bytes;
use futures::StreamExt;
use grafbase_telemetry::grafbase_client::Client;
use runtime::auth::AccessToken;
use std::{future::Future, sync::Arc};

use crate::{
    graphql_over_http::{Http, ResponseFormat},
    request::{BatchRequest, QueryParamsRequest, Request},
    response::{GraphqlError, Response},
    Body, ErrorCode,
};

use super::{runtime::HooksContext, Engine, Runtime, RuntimeExt};

mod prepare;
mod single;
mod stream;

pub(crate) use stream::StreamResponse;

pub(crate) struct RequestContext {
    /// Indicates whether mutations are allowed in the current request context.
    pub mutations_allowed: bool,
    /// The HTTP headers received with the request.
    pub headers: http::HeaderMap,
    /// The format in which the response should be sent.
    pub response_format: ResponseFormat,
    /// The client instance that made the request, if available.
    pub client: Option<Client>,
    /// The access token used for authentication in the request.
    pub access_token: AccessToken,
}

impl<R: Runtime> Engine<R> {
    /// Unpacks an HTTP request into its constituent parts, including the request
    /// headers, body, and response format.
    ///
    /// This function validates that the request method is either GET or POST and
    /// checks that the Content-Type header is set to a supported type for POST
    /// requests. If the request is well-formed, it returns a tuple containing
    /// the request parts, body, and the determined response format.
    ///
    /// # Parameters
    ///
    /// - `request`: The HTTP request to unpack, with a body of type `B`.
    ///
    /// # Returns
    ///
    /// - A `Result` containing a tuple of:
    ///   - `http::request::Parts`: The parts of the HTTP request (headers, method, etc.).
    ///   - `B`: The body of the request.
    ///   - `ResponseFormat`: The format in which the response should be sent.
    ///
    /// # Errors
    ///
    /// - Returns an `http::Response<Body>` if the request is invalid, including:
    ///   - 406 Not Acceptable if no supported media type is specified in the Accept header.
    ///   - 415 Unsupported Media Type if the Content-Type header is missing or not of a supported type.
    ///   - 405 Method Not Allowed if the request method is neither GET nor POST.
    pub(super) fn unpack_http_request<B>(
        &self,
        request: http::Request<B>,
    ) -> Result<(http::request::Parts, B, ResponseFormat), http::Response<Body>> {
        let (parts, body) = request.into_parts();

        let Some(format) = ResponseFormat::extract_from(&parts.headers, self.default_response_format) else {
            // GraphQL-over-HTTP spec:
            //   In alignment with the HTTP 1.1 Accept specification, when a client does not include at least one supported media type in the Accept HTTP header, the server MUST either:
            //     1. Respond with a 406 Not Acceptable status code and stop processing the request (RECOMMENDED); OR
            //     2. Disregard the Accept header and respond with the server's choice of media type (NOT RECOMMENDED).
            return Err(Http::error(
                self.default_response_format,
                Response::not_acceptable_error(),
            ));
        };

        if parts.method == http::Method::POST {
            // GraphQL-over-HTTP spec:
            //   If the client does not supply a Content-Type header with a POST request,
            //   the server SHOULD reject the request using the appropriate 4xx status code.
            if !content_type_is_application_json(&parts.headers) {
                return Err(Http::error(format, Response::unsupported_media_type()));
            }
        } else if parts.method != http::Method::GET {
            return Err(Http::error(
                format,
                Response::method_not_allowed("Only GET or POST are supported."),
            ));
        }

        Ok((parts, body, format))
    }

    /// Creates a new request context for the GraphQL request.
    ///
    /// This function performs the necessary steps to construct a `RequestContext`
    /// based on the provided parameters. It extracts the client from the headers,
    /// processes hooks, and authenticates using the access token.
    ///
    /// # Parameters
    ///
    /// - `mutations_allowed`: A boolean indicating whether mutations are allowed.
    /// - `headers`: The HTTP headers received with the request.
    /// - `response_format`: The desired response format for the request.
    ///
    /// # Returns
    ///
    /// A `Result` containing a tuple of:
    /// - `RequestContext`: The context containing details about the request.
    /// - `HooksContext<R>`: The context for the hooks associated with the runtime.
    ///
    /// # Errors
    ///
    /// This function may return a `Response` indicating failure in the following scenarios:
    /// - If authentication fails, an unauthenticated response is returned.
    /// - If the rate limit has been exceeded, a rate-limited response is returned.
    /// - If the on-gateway-request hook returns an error.
    pub(super) async fn create_request_context(
        &self,
        mutations_allowed: bool,
        headers: http::HeaderMap,
        response_format: ResponseFormat,
    ) -> Result<(RequestContext, HooksContext<R>), Response> {
        let client = Client::extract_from(&headers);

        let (hooks_context, headers) = self
            .runtime
            .hooks()
            .on_gateway_request(headers)
            .await
            .map_err(|ErrorResponse { status, error }| Response::refuse_request_with(status, error))?;

        let Some(access_token) = self.auth.authenticate(&headers).await else {
            return Err(Response::unauthenticated());
        };

        // Currently it doesn't rely on authentication, but likely will at some point.
        if self.runtime.rate_limiter().limit(&RateLimitKey::Global).await.is_err() {
            return Err(Response::gateway_rate_limited());
        }

        Ok((
            RequestContext {
                mutations_allowed,
                headers,
                response_format,
                client,
                access_token,
            },
            hooks_context,
        ))
    }

    /// Extracts a well-formed GraphQL-over-HTTP request from the provided method,
    /// URI, response format, and body. This function handles both POST and GET
    /// methods according to the GraphQL-over-HTTP specification.
    ///
    /// For POST requests, it awaits the body, ensuring that it is valid JSON and
    /// properly deserializes it into a GraphQL operation. If the body cannot be
    /// deserialized, it returns an error response with details about the failure.
    ///
    /// For GET requests, it retrieves the query parameters from the URI and
    /// deserializes them into a GraphQL operation. If deserialization fails,
    /// an appropriate error response is returned.
    ///
    /// # Parameters
    ///
    /// - `method`: The HTTP method used for the request (GET or POST).
    /// - `uri`: The URI from which to extract query parameters if the request is a GET.
    /// - `response_format`: The desired format for the response.
    /// - `body`: A `Future` representing the asynchronous operation to retrieve the request body.
    ///
    /// # Returns
    ///
    /// A `Result` containing either:
    /// - `BatchRequest`: The successfully extracted and deserialized request.
    ///
    /// Or an error response containing details about the failure.
    pub(super) async fn extract_well_formed_graphql_over_http_request<F>(
        &self,
        method: http::method::Method,
        uri: http::Uri,
        response_format: ResponseFormat,
        body: F,
    ) -> Result<BatchRequest, http::Response<Body>>
    where
        F: Future<Output = Result<Bytes, (http::StatusCode, String)>> + Send,
    {
        if method == http::Method::POST {
            let body = body.await.map_err(|(status, message)| {
                Http::error(
                    response_format,
                    Response::refuse_request_with(status, GraphqlError::new(message, ErrorCode::BadRequest)),
                )
            })?;

            self.runtime.metrics().record_request_body_size(body.len());

            serde_json::from_slice(&body).map_err(|err| {
                Http::error(
                    response_format,
                    Response::not_well_formed_graphql_over_http_request(format_args!(
                        "JSON deserialization failure: {err}",
                    )),
                )
            })
        } else {
            let query = uri.query().unwrap_or_default();

            serde_urlencoded::from_str::<QueryParamsRequest>(query)
                .map(|request| BatchRequest::Single(request.into()))
                .map_err(|err| {
                    Http::error(
                        response_format,
                        Response::not_well_formed_graphql_over_http_request(format_args!(
                            "Could not deserialize request from query parameters: {err}"
                        )),
                    )
                })
        }
    }

    /// Executes a well-formed GraphQL request extracted from the GraphQL-over-HTTP
    /// request. This function supports both single queries and batch queries.
    ///
    /// Depending on the response format specified in the `RequestContext`, it can
    /// return either a streamed response or a complete response for a single request.
    ///
    /// # Parameters
    ///
    /// - `request_context`: The context containing details about the request.
    /// - `hooks_context`: The context for the hooks associated with the runtime.
    /// - `request`: The GraphQL request to execute, which can be either a single
    ///   request or a batch of requests.
    ///
    /// # Returns
    ///
    /// - An `http::Response<Body>` containing the result of the executed GraphQL request.
    ///
    /// # Errors
    ///
    /// This function will handle timeouts and may return an error response when:
    /// - The specified timeout is exceeded while processing the request.
    pub(super) async fn execute_well_formed_graphql_request(
        self: &Arc<Self>,
        request_context: RequestContext,
        hooks_context: HooksContext<R>,
        request: BatchRequest,
    ) -> http::Response<Body> {
        match request {
            BatchRequest::Single(request) => match request_context.response_format {
                ResponseFormat::Streaming(format) => {
                    Http::stream(
                        format,
                        hooks_context.clone(),
                        self.execute_stream(Arc::new(request_context), hooks_context, request),
                    )
                    .await
                }
                ResponseFormat::Complete(format) => {
                    let Some(response) = self
                        .runtime
                        .with_timeout(
                            self.schema.settings.timeout,
                            self.execute_single(&request_context, hooks_context.clone(), request),
                        )
                        .await
                    else {
                        return Http::error(request_context.response_format, Response::gateway_timeout());
                    };

                    Http::single(format, hooks_context, response)
                }
            },
            BatchRequest::Batch(requests) => {
                let ResponseFormat::Complete(format) = request_context.response_format else {
                    return Http::error(
                        request_context.response_format,
                        Response::bad_request_but_well_formed_graphql_over_http_request(
                            "batch requests cannot be returned as multipart or event-stream responses",
                        ),
                    );
                };

                self.runtime.metrics().record_batch_size(requests.len());

                let Some(responses) = self
                    .runtime
                    .with_timeout(
                        self.schema.settings.timeout,
                        futures_util::stream::iter(requests.into_iter())
                            .then(|request| self.execute_single(&request_context, hooks_context.clone(), request))
                            .collect::<Vec<_>>(),
                    )
                    .await
                else {
                    return Http::error(request_context.response_format, Response::gateway_timeout());
                };

                Http::batch(format, hooks_context, responses)
            }
        }
    }

    /// Executes a well-formed GraphQL request over a WebSocket connection.
    ///
    /// This function manages the execution of a GraphQL request sent through
    /// a WebSocket, enabling real-time data communication. The request is
    /// handled asynchronously, and the response is streamed back to the client.
    ///
    /// # Parameters
    ///
    /// - `request_context`: Contains details about the request, including headers
    ///   and client information.
    /// - `hooks_context`: The context for the hooks associated with the runtime.
    /// - `request`: The GraphQL request to execute over the WebSocket.
    ///
    /// # Returns
    ///
    /// A `StreamResponse` containing the result of the executed GraphQL request,
    /// which may be sent to the client over the WebSocket connection.
    pub(super) fn execute_websocket_well_formed_graphql_request(
        self: &Arc<Self>,
        request_context: Arc<RequestContext>,
        hooks_context: HooksContext<R>,
        request: Request,
    ) -> StreamResponse {
        self.execute_stream(request_context, hooks_context, request)
    }
}

fn content_type_is_application_json(headers: &http::HeaderMap) -> bool {
    static APPLICATION_JSON: http::HeaderValue = http::HeaderValue::from_static("application/json");

    let Some(header) = headers.get(http::header::CONTENT_TYPE) else {
        return false;
    };

    let header = header.to_str().unwrap_or_default();
    let (without_parameters, _) = header.split_once(';').unwrap_or((header, ""));

    without_parameters == APPLICATION_JSON
}
