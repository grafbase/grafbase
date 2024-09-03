use ::runtime::{error::ErrorResponse, hooks::Hooks, rate_limiting::RateLimitKey};
use bytes::Bytes;
use futures::StreamExt;
use futures_util::Stream;
use grafbase_telemetry::grafbase_client::Client;
use runtime::auth::AccessToken;
use std::{future::Future, sync::Arc};

use crate::{
    graphql_over_http::{Http, ResponseFormat},
    request::{BatchRequest, QueryParamsRequest, Request},
    response::{GraphqlError, RefusedRequestResponse, RequestErrorResponse, Response},
    Body, ErrorCode,
};

use super::{Engine, Runtime, RuntimeExt};

mod prepare;
mod single;
mod stream;

pub(crate) struct RequestContext<C> {
    pub mutations_allowed: bool,
    pub headers: http::HeaderMap,
    pub response_format: ResponseFormat,
    pub client: Option<Client>,
    pub access_token: AccessToken,
    pub hooks_context: C,
}

impl<R: Runtime> Engine<R> {
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
            return Err(Http::from(
                self.default_response_format,
                RefusedRequestResponse::not_acceptable_error(),
            ));
        };

        if parts.method == http::Method::POST {
            // GraphQL-over-HTTP spec:
            //   If the client does not supply a Content-Type header with a POST request,
            //   the server SHOULD reject the request using the appropriate 4xx status code.
            if !content_type_is_application_json(&parts.headers) {
                return Err(Http::from(format, RefusedRequestResponse::unsupported_media_type()));
            }
        } else if parts.method != http::Method::GET {
            return Err(Http::from(
                format,
                RefusedRequestResponse::method_not_allowed("Only GET or POST are supported."),
            ));
        }

        Ok((parts, body, format))
    }

    pub(super) async fn create_request_context(
        &self,
        mutations_allowed: bool,
        headers: http::HeaderMap,
        response_format: ResponseFormat,
    ) -> Result<RequestContext<<R::Hooks as Hooks>::Context>, Response> {
        let client = Client::extract_from(&headers);

        let (hooks_context, headers) = self
            .runtime
            .hooks()
            .on_gateway_request(headers)
            .await
            .map_err(|ErrorResponse { status, error }| Response::refuse_request_with(status, error))?;

        let Some(access_token) = self.auth.authenticate(&headers).await else {
            return Err(RefusedRequestResponse::unauthenticated());
        };

        // Currently it doesn't rely on authentication, but likely will at some point.
        if self.runtime.rate_limiter().limit(&RateLimitKey::Global).await.is_err() {
            return Err(RefusedRequestResponse::gateway_rate_limited());
        }

        Ok(RequestContext {
            mutations_allowed,
            headers,
            response_format,
            client,
            access_token,
            hooks_context,
        })
    }

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
                Http::from(
                    response_format,
                    Response::refuse_request_with(status, GraphqlError::new(message, ErrorCode::BadRequest)),
                )
            })?;

            self.runtime.metrics().record_request_body_size(body.len());

            serde_json::from_slice(&body).map_err(|err| {
                Http::from(
                    response_format,
                    RefusedRequestResponse::not_well_formed_graphql_over_http_request(format_args!(
                        "JSON deserialization failure: {err}",
                    )),
                )
            })
        } else {
            let query = uri.query().unwrap_or_default();

            serde_urlencoded::from_str::<QueryParamsRequest>(query)
                .map(|request| BatchRequest::Single(request.into()))
                .map_err(|err| {
                    Http::from(
                        response_format,
                        RefusedRequestResponse::not_well_formed_graphql_over_http_request(format_args!(
                            "Could not deserialize request from query parameters: {err}"
                        )),
                    )
                })
        }
    }

    pub(super) async fn execute_well_formed_graphql_request(
        self: &Arc<Self>,
        request_context: RequestContext<<R::Hooks as Hooks>::Context>,
        request: BatchRequest,
    ) -> http::Response<Body> {
        match request {
            BatchRequest::Single(request) => {
                if let ResponseFormat::Streaming(format) = request_context.response_format {
                    Http::stream(format, self.execute_stream(Arc::new(request_context), request)).await
                } else {
                    let Some((response, operation_hook_result)) = self
                        .runtime
                        .with_timeout(
                            self.schema.settings.timeout,
                            self.execute_single(&request_context, request),
                        )
                        .await
                    else {
                        return Http::from(request_context.response_format, RequestErrorResponse::gateway_timeout());
                    };

                    let mut response = Http::from(request_context.response_format, response);

                    if let Some(result) = operation_hook_result {
                        response.extensions_mut().insert(vec![result]);
                    };

                    response
                }
            }
            BatchRequest::Batch(requests) => {
                let ResponseFormat::Complete(format) = request_context.response_format else {
                    return Http::from(
                        request_context.response_format,
                        RequestErrorResponse::bad_request_but_well_formed_graphql_over_http_request(
                            "batch requests cannot be returned as multipart or event-stream responses",
                        ),
                    );
                };

                self.runtime.metrics().record_batch_size(requests.len());

                let Some((responses, operation_hook_results)): Option<(Vec<_>, Vec<_>)> = self
                    .runtime
                    .with_timeout(
                        self.schema.settings.timeout,
                        futures_util::stream::iter(requests.into_iter())
                            .then(|request| self.execute_single(&request_context, request))
                            .unzip(),
                    )
                    .await
                else {
                    return Http::from(request_context.response_format, RequestErrorResponse::gateway_timeout());
                };

                let mut response = Http::batch(format, responses);

                response
                    .extensions_mut()
                    .insert(operation_hook_results.into_iter().flatten().collect::<Vec<_>>());

                response
            }
        }
    }

    pub(super) fn execute_websocket_well_formed_graphql_request(
        self: &Arc<Self>,
        request_context: Arc<RequestContext<<R::Hooks as Hooks>::Context>>,
        request: Request,
    ) -> impl Stream<Item = Response> + Send + 'static {
        self.execute_stream(request_context, request)
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
