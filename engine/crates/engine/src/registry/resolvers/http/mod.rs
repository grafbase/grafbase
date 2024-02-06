use std::{collections::BTreeMap, pin::Pin};

use async_runtime::make_send_on_wasm;
use futures_util::Future;
use reqwest::Url;

use self::parameters::ParamApply;
use super::{ResolvedValue, ResolverContext};
use crate::{registry::variables::VariableResolveDefinition, Context, ContextExt, ContextField, Error, RequestHeaders};

mod parameters;

#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub struct HttpResolver {
    pub method: String,
    pub url: String,
    pub api_name: String,
    pub path_parameters: Vec<PathParameter>,
    pub query_parameters: Vec<QueryParameter>,
    pub request_body: Option<RequestBody>,
    pub expected_status: ExpectedStatusCode,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub struct PathParameter {
    pub name: String,
    pub variable_resolve_definition: VariableResolveDefinition,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub struct QueryParameter {
    pub name: String,
    pub variable_resolve_definition: VariableResolveDefinition,
    pub encoding_style: QueryParameterEncodingStyle,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub struct RequestBody {
    pub variable_resolve_definition: VariableResolveDefinition,
    pub content_type: RequestBodyContentType,
}

#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub enum QueryParameterEncodingStyle {
    Form,
    FormExploded,
    DeepObject,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub enum RequestBodyContentType {
    Json,
    FormEncoded(BTreeMap<String, QueryParameterEncodingStyle>),
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub enum ExpectedStatusCode {
    Exact(u16),
    Range(std::ops::Range<u16>),
}

impl HttpResolver {
    pub fn resolve<'a>(
        &'a self,
        ctx: &'a ContextField<'_>,
        _resolver_ctx: &ResolverContext<'a>,
        last_resolver_value: Option<ResolvedValue>,
    ) -> Pin<Box<dyn Future<Output = Result<ResolvedValue, Error>> + Send + 'a>> {
        let last_resolver_value = last_resolver_value.map(ResolvedValue::take);

        let request_headers = ctx.data::<RequestHeaders>().ok();

        let headers = ctx
            .registry()
            .http_headers
            .get(&self.api_name)
            .zip(request_headers)
            .map(|(connector_headers, request_headers)| connector_headers.build_header_vec(request_headers))
            .unwrap_or_default();

        Box::pin(make_send_on_wasm(async move {
            let runtime_ctx = ctx.data::<runtime::Context>()?;
            let fetch_log_endpoint_url = runtime_ctx.log.fetch_log_endpoint_url.as_deref();
            let ray_id = &runtime_ctx.ray_id();
            let url = self.build_url(ctx, last_resolver_value.as_ref())?;
            let mut request_builder = reqwest::Client::new().request(self.method.parse()?, Url::parse(&url)?);

            for (name, value) in headers {
                request_builder = request_builder.header(name, value);
            }

            if let Some(request_body) = &self.request_body {
                let variable = request_body
                    .variable_resolve_definition
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

            if !self.expected_status.contains(response.status()) {
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
        &self,
        ctx: &ContextField<'_>,
        last_resolver_value: Option<&serde_json::Value>,
    ) -> Result<String, Error> {
        let mut url = self.url.clone();

        for param in &self.path_parameters {
            let variable = param.variable_resolve_definition.resolve(ctx, last_resolver_value)?;

            url = url.apply_path_parameter(param, variable)?;
        }

        let query_variables = self
            .query_parameters
            .iter()
            .map(|param| param.variable_resolve_definition.resolve(ctx, last_resolver_value))
            .collect::<Result<Vec<_>, _>>()?;

        url.apply_query_parameters(&self.query_parameters, &query_variables)
    }
}

impl ExpectedStatusCode {
    pub fn is_success(&self) -> bool {
        match self {
            ExpectedStatusCode::Exact(code) => 200 <= *code && *code < 300,
            ExpectedStatusCode::Range(code_range) => code_range.contains(&200) && code_range.end < 300,
        }
    }

    pub fn contains(&self, code: reqwest::StatusCode) -> bool {
        match self {
            ExpectedStatusCode::Exact(expected_status) => code.as_u16() == *expected_status,
            ExpectedStatusCode::Range(range) => range.contains(&code.as_u16()),
        }
    }
}
