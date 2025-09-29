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
pub(crate) use stream::*;

use ::runtime::rate_limiting::RateLimitKey;
use bytes::Bytes;
use grafbase_telemetry::grafbase_client::Client;
use operation::{BatchRequest, QueryParamsRequest};
use std::{future::Future, sync::Arc};

use crate::{
    Body, ContractAwareEngine, Engine, RequestExtensions, Runtime,
    graphql_over_http::{ContentType, ResponseFormat},
    mcp::McpRequestContext,
    response::Response,
    websocket::InitPayload,
};

impl<R: Runtime> ContractAwareEngine<R> {
    #[allow(clippy::result_large_err)]
    pub(crate) fn unpack_http_request<B>(&self, request: http::Request<B>) -> Result<(Parts, B), http::Response<Body>> {
        let (mut parts, body) = request.into_parts();

        let Some(response_format) = ResponseFormat::extract_from(&parts.headers) else {
            // GraphQL-over-HTTP spec:
            //   In alignment with the HTTP 1.1 Accept specification, when a client does not include at least one supported media type in the Accept HTTP header, the server MUST either:
            //     1. Respond with a 406 Not Acceptable status code and stop processing the request (RECOMMENDED); OR
            //     2. Disregard the Accept header and respond with the server's choice of media type (NOT RECOMMENDED).
            return Err(errors::not_acceptable_error(
                self.no_contract.schema.config.error_code_mapping.clone(),
                ResponseFormat::default(),
            ));
        };

        let content_type = if parts.method == http::Method::POST {
            // GraphQL-over-HTTP spec:
            //   If the client does not supply a Content-Type header with a POST request,
            //   the server SHOULD reject the request using the appropriate 4xx status code.
            ContentType::extract(&parts.headers).ok_or_else(|| {
                errors::unsupported_content_type(
                    self.no_contract.schema.config.error_code_mapping.clone(),
                    response_format,
                )
            })?
        } else {
            if parts.method != http::Method::GET {
                return Err(errors::method_not_allowed(
                    self.no_contract.schema.config.error_code_mapping.clone(),
                    response_format,
                    "Only GET or POST are supported.",
                ));
            }
            ContentType::Json
        };

        // Config doesn't depend on the contract.
        let include_grafbase_response_extension =
            should_include_grafbase_response_extension(&self.no_contract.schema.config, &parts.headers);

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

        let parts = Parts {
            ctx,
            headers: parts.headers,
            extensions: parts.extensions.remove().expect("Missing request extensions"),
        };

        Ok((parts, body))
    }
}

pub(crate) struct Parts {
    pub ctx: EarlyHttpContext,
    pub headers: http::HeaderMap,
    pub extensions: RequestExtensions,
}

impl<R: Runtime> Engine<R> {
    pub(crate) async fn create_graphql_context(
        self: &Arc<Self>,
        ctx: &EarlyHttpContext,
        headers: http::HeaderMap,
        extensions: RequestExtensions,
        websocket_init_payload: Option<InitPayload>,
    ) -> Result<Arc<RequestContext>, Response> {
        let client = Client::extract_from(&headers);

        // Currently it doesn't rely on authentication, but likely will at some point.
        if self.runtime.rate_limiter().limit(&RateLimitKey::Global).await.is_err() {
            return Err(errors::response::gateway_rate_limited(
                self.schema.config.error_code_mapping.clone(),
            ));
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
            token: extensions.token,
            subgraph_default_headers,
            include_grafbase_response_extension: ctx.include_grafbase_response_extension,
            include_mcp_response_extension: ctx.include_mcp_response_extension,
            event_queue: extensions.event_queue,
            hooks_context: extensions.hooks_context,
        };

        Ok(Arc::new(request_context))
    }

    pub(crate) async fn extract_well_formed_graphql_over_http_request<F>(
        &self,
        ctx: &EarlyHttpContext,
        body: F,
    ) -> Result<BatchRequest, Response>
    where
        F: Future<Output = Result<Bytes, (http::StatusCode, String)>> + Send,
    {
        if ctx.method == http::Method::POST {
            let body = body.await.map_err(|(status_code, message)| {
                errors::refuse_request_with(self.schema.config.error_code_mapping.clone(), status_code, message)
            })?;

            self.runtime.metrics().record_request_body_size(body.len());

            match ctx.content_type {
                ContentType::Json => sonic_rs::from_slice(&body).map_err(|err| {
                    errors::not_well_formed_graphql_over_http_request(
                        self.schema.config.error_code_mapping.clone(),
                        format_args!("JSON deserialization failure: {err}",),
                    )
                }),
                ContentType::Cbor => minicbor_serde::from_slice(&body).map_err(|err| {
                    errors::not_well_formed_graphql_over_http_request(
                        self.schema.config.error_code_mapping.clone(),
                        format_args!("CBOR deserialization failure: {err}"),
                    )
                }),
            }
        } else {
            let query = ctx.uri.query().unwrap_or_default();

            serde_urlencoded::from_str::<QueryParamsRequest>(query)
                .map(|request| BatchRequest::Single(request.into()))
                .map_err(|err| {
                    errors::not_well_formed_graphql_over_http_request(
                        self.schema.config.error_code_mapping.clone(),
                        format_args!("Could not deserialize request from query parameters: {err}"),
                    )
                })
        }
    }
}
