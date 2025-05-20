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
    Body, Engine, Runtime,
    engine::WasmContext,
    graphql_over_http::{ContentType, ResponseFormat},
    mcp::McpRequestContext,
    response::Response,
    websocket::InitPayload,
};

impl<R: Runtime> Engine<R> {
    #[allow(clippy::result_large_err)]
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

        let content_type = if parts.method == http::Method::POST {
            // GraphQL-over-HTTP spec:
            //   If the client does not supply a Content-Type header with a POST request,
            //   the server SHOULD reject the request using the appropriate 4xx status code.
            ContentType::extract(&parts.headers).ok_or_else(|| errors::unsupported_content_type(response_format))?
        } else {
            if parts.method != http::Method::GET {
                return Err(errors::method_not_allowed(
                    response_format,
                    "Only GET or POST are supported.",
                ));
            }
            ContentType::Json
        };

        let include_grafbase_response_extension =
            should_include_grafbase_response_extension(&self.schema, &parts.headers);
        let mut ctx = EarlyHttpContext {
            can_mutate: !parts.method.is_safe(),
            method: parts.method,
            uri: parts.uri,
            response_format,
            content_type,
            include_grafbase_response_extension,
            include_mcp_response_extension: false,
        };
        if let Some(mcp) = parts.extensions.get::<McpRequestContext>() {
            ctx.can_mutate &= mcp.execute_mutations;
            ctx.include_mcp_response_extension = true;
        }

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
            can_mutate: ctx.can_mutate,
            headers,
            response_format: ctx.response_format,
            client,
            token,
            subgraph_default_headers,
            include_grafbase_response_extension: ctx.include_grafbase_response_extension,
            include_mcp_response_extension: ctx.include_mcp_response_extension,
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

            match ctx.content_type {
                ContentType::Json => sonic_rs::from_slice(&body).map_err(|err| {
                    errors::not_well_formed_graphql_over_http_request(format_args!(
                        "JSON deserialization failure: {err}",
                    ))
                }),
                ContentType::Cbor => minicbor_serde::from_slice(&body).map_err(|err| {
                    errors::not_well_formed_graphql_over_http_request(format_args!(
                        "CBOR deserialization failure: {err}"
                    ))
                }),
            }
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
