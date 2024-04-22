use std::pin::Pin;

use async_runtime::make_send_on_wasm;
use futures_util::Future;
use registry_v2::resolvers::http::*;
use reqwest::Url;

use self::parameters::ParamApply;
use super::{ResolvedValue, ResolverContext};
use crate::{
    registry::{connector_headers::build_connector_header_vec, variables::VariableResolveDefinition},
    Context, ContextExt, ContextField, Error, RequestHeaders,
};

mod parameters;

pub fn resolve<'a>(
    resolver: &'a HttpResolver,
    ctx: &'a ContextField<'_>,
    _resolver_ctx: &ResolverContext<'a>,
    last_resolver_value: Option<ResolvedValue>,
) -> Pin<Box<dyn Future<Output = Result<ResolvedValue, Error>> + Send + 'a>> {
    let last_resolver_value = last_resolver_value.map(ResolvedValue::take);

    let request_headers = ctx.data::<RequestHeaders>().ok();

    let headers = ctx
        .registry()
        .http_headers
        .get(&resolver.api_name)
        .zip(request_headers)
        .map(|(connector_headers, request_headers)| build_connector_header_vec(connector_headers, request_headers))
        .unwrap_or_default();

    Box::pin(make_send_on_wasm(async move {
        let runtime_ctx = ctx.data::<runtime::Context>()?;
        let fetch_log_endpoint_url = runtime_ctx.log.fetch_log_endpoint_url.as_deref();
        let ray_id = &runtime_ctx.ray_id();
        let url = build_url(resolver, ctx, last_resolver_value.as_ref())?;
        let mut request_builder = reqwest::Client::new()
            .request(resolver.method.parse()?, Url::parse(&url)?)
            .timeout(std::time::Duration::from_secs(30));

        for (name, value) in headers {
            request_builder = request_builder.header(name, value);
        }

        if let Some(request_body) = &resolver.request_body {
            let variable = VariableResolveDefinition::from_registry_v2(
                request_body.variable_resolve_definition.clone(),
                ctx.registry(),
            )
            .resolve::<serde_json::Value>(ctx, last_resolver_value)?;

            match &request_body.content_type {
                RequestBodyContentType::Json => {
                    request_builder = request_builder.json(&variable);
                }
                RequestBodyContentType::FormEncoded(encoding_styles) => {
                    request_builder =
                        request_builder.header(reqwest::header::CONTENT_TYPE, "application/x-www-form-urlencoded");
                    request_builder =
                        request_builder.body(String::new().apply_body_parameters(encoding_styles, variable)?);
                }
            }
        }

        let response = super::logged_fetch::send_logged_request(ray_id, fetch_log_endpoint_url, request_builder)
            .await
            .map_err(|e| Error::new(e.to_string()))?;

        if !status_contains(&resolver.expected_status, response.status()) {
            return Err(Error::new(format!(
                "Received an unexpected status from the downstream server: {}",
                response.status(),
            )));
        }

        let data = response
            .json::<serde_json::Value>()
            .await
            .map_err(|e| Error::new(e.to_string()))?;

        Ok(ResolvedValue::new(data))
    }))
}

fn build_url(
    resolver: &HttpResolver,
    ctx: &ContextField<'_>,
    last_resolver_value: Option<&serde_json::Value>,
) -> Result<String, Error> {
    let mut url = resolver.url.clone();

    for param in &resolver.path_parameters {
        let variable =
            VariableResolveDefinition::from_registry_v2(param.variable_resolve_definition.clone(), ctx.registry())
                .resolve(ctx, last_resolver_value)?;

        url = url.apply_path_parameter(param, variable)?;
    }

    let query_variables = resolver
        .query_parameters
        .iter()
        .map(|param| {
            VariableResolveDefinition::from_registry_v2(param.variable_resolve_definition.clone(), ctx.registry())
                .resolve(ctx, last_resolver_value)
        })
        .collect::<Result<Vec<_>, _>>()?;

    url.apply_query_parameters(&resolver.query_parameters, &query_variables)
}

pub fn status_is_success(status: &ExpectedStatusCode) -> bool {
    match status {
        ExpectedStatusCode::Exact(code) => 200 <= *code && *code < 300,
        ExpectedStatusCode::Range(code_range) => code_range.contains(&200) && code_range.end < 300,
    }
}

pub fn status_contains(status: &ExpectedStatusCode, code: reqwest::StatusCode) -> bool {
    match status {
        ExpectedStatusCode::Exact(expected_status) => code.as_u16() == *expected_status,
        ExpectedStatusCode::Range(range) => range.contains(&code.as_u16()),
    }
}
