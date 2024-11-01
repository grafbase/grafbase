use ::runtime::{error::ErrorResponse, hooks::Hooks, rate_limiting::RateLimitKey};
use bytes::Bytes;
use futures::StreamExt;
use grafbase_telemetry::grafbase_client::Client;
use runtime::auth::AccessToken;
use std::{future::Future, sync::Arc};

use crate::{
    graphql_over_http::{Http, ResponseFormat},
    request::{BatchRequest, QueryParamsRequest, Request},
    response::Response,
    Body,
};

use super::{errors, runtime::HooksContext, Engine, Runtime, RuntimeExt};

mod prepare;
mod single;
mod stream;

pub(crate) use stream::StreamResponse;

pub(crate) struct RequestContext {
    pub mutations_allowed: bool,
    pub headers: http::HeaderMap,
    pub response_format: ResponseFormat,
    pub client: Option<Client>,
    pub access_token: AccessToken,
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
            return Err(errors::not_acceptable_error(self.default_response_format));
        };

        if parts.method == http::Method::POST {
            // GraphQL-over-HTTP spec:
            //   If the client does not supply a Content-Type header with a POST request,
            //   the server SHOULD reject the request using the appropriate 4xx status code.
            if !content_type_is_application_json(&parts.headers) {
                return Err(errors::unsupported_media_type(format));
            }
        } else if parts.method != http::Method::GET {
            return Err(errors::method_not_allowed(format, "Only GET or POST are supported."));
        }

        Ok((parts, body, format))
    }

    pub(super) async fn create_request_context(
        &self,
        mutations_allowed: bool,
        headers: http::HeaderMap,
        response_format: ResponseFormat,
    ) -> Result<
        (RequestContext, HooksContext<R>),
        (
            Response<<R::Hooks as Hooks>::OnOperationResponseOutput>,
            HooksContext<R>,
        ),
    > {
        let client = Client::extract_from(&headers);

        let (hooks_context, headers) = self.runtime.hooks().on_gateway_request(headers).await.map_err(
            |(context, ErrorResponse { status, error })| (Response::refuse_request_with(status, error), context),
        )?;

        let Some(access_token) = self.auth.authenticate(&headers).await else {
            return Err((errors::response::unauthenticated(), hooks_context));
        };

        // Currently it doesn't rely on authentication, but likely will at some point.
        if self.runtime.rate_limiter().limit(&RateLimitKey::Global).await.is_err() {
            return Err((errors::response::gateway_rate_limited(), hooks_context));
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
            let body = body
                .await
                .map_err(|(status_code, message)| errors::refuse_request_with(response_format, status_code, message))?;

            self.runtime.metrics().record_request_body_size(body.len());

            serde_json::from_slice(&body).map_err(|err| {
                errors::not_well_formed_graphql_over_http_request(
                    response_format,
                    format_args!("JSON deserialization failure: {err}",),
                )
            })
        } else {
            let query = uri.query().unwrap_or_default();

            serde_urlencoded::from_str::<QueryParamsRequest>(query)
                .map(|request| BatchRequest::Single(request.into()))
                .map_err(|err| {
                    errors::not_well_formed_graphql_over_http_request(
                        response_format,
                        format_args!("Could not deserialize request from query parameters: {err}"),
                    )
                })
        }
    }

    pub(super) async fn execute_well_formed_graphql_request(
        self: &Arc<Self>,
        request_context: RequestContext,
        hooks_context: HooksContext<R>,
        request: BatchRequest,
    ) -> http::Response<Body> {
        let request_context = Arc::new(request_context);
        match request {
            BatchRequest::Single(request) => match request_context.response_format {
                ResponseFormat::Streaming(format) => {
                    Http::stream(
                        format,
                        hooks_context.clone(),
                        self.execute_stream(request_context, hooks_context, request),
                    )
                    .await
                }
                ResponseFormat::Complete(format) => {
                    let Some(response) = self
                        .with_gateway_timeout(self.execute_single(&request_context, hooks_context.clone(), request))
                        .await
                    else {
                        return errors::gateway_timeout(request_context.response_format);
                    };

                    Http::single(format, hooks_context, response)
                }
            },
            BatchRequest::Batch(requests) => {
                let ResponseFormat::Complete(format) = request_context.response_format else {
                    return errors::bad_request_but_well_formed_graphql_over_http_request(
                        request_context.response_format,
                        "batch requests cannot be returned as multipart or event-stream responses",
                    );
                };

                if !self.schema.settings.batching.enabled {
                    return errors::bad_request_but_well_formed_graphql_over_http_request(
                        request_context.response_format,
                        "batching is not enabled for this service",
                    );
                }

                if let Some(limit) = self.schema.settings.batching.limit {
                    if requests.len() > limit {
                        return errors::bad_request_but_well_formed_graphql_over_http_request(
                            request_context.response_format,
                            format_args!("batch size exceeds limit of {}", limit),
                        );
                    }
                }

                self.runtime.metrics().record_batch_size(requests.len());

                let Some(responses) = self
                    .with_gateway_timeout(
                        futures_util::stream::iter(requests.into_iter())
                            .then(|request| self.execute_single(&request_context, hooks_context.clone(), request))
                            .collect::<Vec<_>>(),
                    )
                    .await
                else {
                    return errors::gateway_timeout(request_context.response_format);
                };

                Http::batch(format, hooks_context, responses)
            }
        }
    }

    pub(super) fn execute_websocket_well_formed_graphql_request(
        self: &Arc<Self>,
        request_context: Arc<RequestContext>,
        hooks_context: HooksContext<R>,
        request: Request,
    ) -> StreamResponse<<R::Hooks as Hooks>::OnOperationResponseOutput> {
        self.execute_stream(request_context, hooks_context, request)
    }

    pub(super) async fn with_gateway_timeout<T>(&self, fut: impl Future<Output = T> + Send) -> Option<T> {
        self.runtime.with_timeout(self.schema.settings.timeout, fut).await
    }
}

fn content_type_is_application_json(headers: &http::HeaderMap) -> bool {
    const APPLICATION_JSON: http::HeaderValue = http::HeaderValue::from_static("application/json");

    let Some(header) = headers.get(http::header::CONTENT_TYPE) else {
        return false;
    };

    let header = header.to_str().unwrap_or_default();
    let (without_parameters, _) = header.split_once(';').unwrap_or((header, ""));

    without_parameters == APPLICATION_JSON
}
