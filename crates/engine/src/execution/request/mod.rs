mod context;
pub(crate) mod errors;
mod header_rule;
mod response_extension;
mod single;
mod stream;
mod well_formed_graphql_request;

pub(crate) use context::*;
pub(crate) use header_rule::*;
use response_extension::should_include_grafbase_response_extension;
pub(crate) use response_extension::*;
use runtime::authentication::Authenticate as _;
pub(crate) use stream::*;

use ::runtime::{hooks::Hooks, rate_limiting::RateLimitKey};
use bytes::Bytes;
use error::ErrorResponse;
use grafbase_telemetry::grafbase_client::Client;
use operation::{BatchRequest, QueryParamsRequest};
use std::{future::Future, sync::Arc};

use crate::{
    Body, Engine, Runtime, engine::WasmContext, graphql_over_http::ResponseFormat, response::Response,
    websocket::InitPayload,
};

impl<R: Runtime> Engine<R> {
    pub(crate) fn unpack_http_request<B>(
        &self,
        request: http::Request<B>,
    ) -> Result<(EarlyHttpContext, http::HeaderMap, B), http::Response<Body>> {
        let (parts, body) = request.into_parts();
        let Some(response_format) = ResponseFormat::extract_from(&parts.headers, self.default_response_format) else {
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
                return Err(errors::unsupported_media_type(response_format));
            }
        } else if parts.method != http::Method::GET {
            return Err(errors::method_not_allowed(
                response_format,
                "Only GET or POST are supported.",
            ));
        }

        let include_grafbase_response_extension =
            should_include_grafbase_response_extension(&self.schema, &parts.headers);
        let ctx = EarlyHttpContext {
            method: parts.method,
            uri: parts.uri,
            response_format,
            include_grafbase_response_extension,
        };

        Ok((ctx, parts.headers, body))
    }

    pub(crate) async fn create_graphql_context(
        self: &Arc<Self>,
        ctx: &EarlyHttpContext,
        headers: http::HeaderMap,
        websocket_init_payload: Option<InitPayload>,
    ) -> Result<
        (Arc<RequestContext>, WasmContext<R>),
        (Response<<R::Hooks as Hooks>::OnOperationResponseOutput>, WasmContext<R>),
    > {
        let (wasm_context, headers) = self
            .runtime
            .hooks()
            .on_gateway_request(&ctx.uri.to_string(), headers)
            .await
            .map_err(|(context, ErrorResponse { status, errors })| {
                (Response::refuse_request_with(status, errors), context)
            })?;

        let client = Client::extract_from(&headers);

        let (headers, token) = match self.runtime.authentication().authenticate(headers).await {
            Ok((headers, token)) => (headers, token),
            Err(resp) => {
                let response = Response::refuse_request_with(resp.status, resp.errors);
                return Err((response, wasm_context));
            }
        };

        // Currently it doesn't rely on authentication, but likely will at some point.
        if self.runtime.rate_limiter().limit(&RateLimitKey::Global).await.is_err() {
            return Err((errors::response::gateway_rate_limited(), wasm_context));
        }

        let mut subgraph_default_headers = http::HeaderMap::new();
        apply_header_rules(
            &headers,
            self.schema.default_header_rules(),
            &mut subgraph_default_headers,
        );
        let request_context = RequestContext {
            websocket_init_payload: websocket_init_payload.and_then(|payload| payload.0),
            mutations_allowed: !ctx.method.is_safe(),
            headers,
            response_format: ctx.response_format,
            client,
            token,
            subgraph_default_headers,
            include_grafbase_response_extension: ctx.include_grafbase_response_extension,
        };

        Ok((Arc::new(request_context), wasm_context))
    }

    pub(crate) async fn extract_well_formed_graphql_over_http_request<F>(
        &self,
        ctx: &EarlyHttpContext,
        body: F,
    ) -> Result<BatchRequest, Response<<R::Hooks as Hooks>::OnOperationResponseOutput>>
    where
        F: Future<Output = Result<Bytes, (http::StatusCode, String)>> + Send,
    {
        if ctx.method == http::Method::POST {
            let body = body
                .await
                .map_err(|(status_code, message)| errors::refuse_request_with(status_code, message))?;

            self.runtime.metrics().record_request_body_size(body.len());

            sonic_rs::from_slice(&body).map_err(|err| {
                errors::not_well_formed_graphql_over_http_request(format_args!("JSON deserialization failure: {err}",))
            })
        } else {
            let query = ctx.uri.query().unwrap_or_default();

            serde_urlencoded::from_str::<QueryParamsRequest>(query)
                .map(|request| BatchRequest::Single(request.into()))
                .map_err(|err| {
                    errors::not_well_formed_graphql_over_http_request(format_args!(
                        "Could not deserialize request from query parameters: {err}"
                    ))
                })
        }
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
