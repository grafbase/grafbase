use std::{borrow::Borrow, collections::BTreeMap, pin::Pin, sync::Arc};

use futures_util::Future;
use reqwest::Url;
use send_wrapper::SendWrapper;

use crate::{registry::variables::VariableResolveDefinition, Context, Error};

use self::parameters::ParamApply;

use super::{ResolvedValue, ResolverContext};

mod parameters;

#[serde_with::minify_field_names]
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
        ctx: &'a Context<'_>,
        _resolver_ctx: &ResolverContext<'a>,
        last_resolver_value: Option<&'a ResolvedValue>,
    ) -> Pin<Box<dyn Future<Output = Result<ResolvedValue, Error>> + Send + 'a>> {
        let last_resolver_value = last_resolver_value.map(|val| val.data_resolved.borrow());

        let headers = ctx
            .registry()
            .http_headers
            .get(&self.api_name)
            .map(Vec::as_slice)
            .unwrap_or(&[]);

        Box::pin(SendWrapper::new(async move {
            let url = self.build_url(ctx, last_resolver_value)?;
            let mut request_builder =
                dbg!(reqwest::Client::new().request(self.method.parse()?, Url::parse(&url)?));

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
                        request_builder = request_builder.header(
                            reqwest::header::CONTENT_TYPE,
                            "application/x-www-form-urlencoded",
                        );
                        request_builder = request_builder
                            .body(String::new().apply_body_parameters(encoding_styles, variable)?);
                    }
                }
            }

            let response = request_builder
                .send()
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

            Ok(ResolvedValue::new(Arc::new(dbg!(data))))
        }))
    }

    fn build_url(
        &self,
        ctx: &Context<'_>,
        last_resolver_value: Option<&serde_json::Value>,
    ) -> Result<String, Error> {
        let mut url = self.url.clone();

        for param in &self.path_parameters {
            let variable = param
                .variable_resolve_definition
                .resolve(ctx, last_resolver_value)?;

            url = url.apply_path_parameter(&param, variable)?;
        }

        let query_variables = self
            .query_parameters
            .iter()
            .map(|param| {
                param
                    .variable_resolve_definition
                    .resolve(ctx, last_resolver_value)
            })
            .collect::<Result<Vec<_>, _>>()?;

        url.apply_query_parameters(&self.query_parameters, &query_variables)
    }
}

impl ExpectedStatusCode {
    pub fn contains(&self, code: reqwest::StatusCode) -> bool {
        match self {
            ExpectedStatusCode::Exact(expected_status) => code.as_u16() == *expected_status,
            ExpectedStatusCode::Range(range) => range.contains(&code.as_u16()),
        }
    }
}
