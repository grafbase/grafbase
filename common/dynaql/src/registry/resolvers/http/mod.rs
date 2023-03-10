use std::{borrow::Borrow, collections::BTreeMap, sync::Arc};

use surf::Url;

use crate::{registry::variables::VariableResolveDefinition, Context, Error};

use self::parameters::ParamApply;

use super::{ResolvedValue, ResolverContext};

mod parameters;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub struct HttpResolver {
    pub method: String,
    pub url: String,
    pub api_name: String,
    pub path_parameters: Vec<PathParameter>,
    pub query_parameters: Vec<QueryParameter>,
    pub request_body: Option<RequestBody>,
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

impl HttpResolver {
    pub async fn resolve(
        &self,
        ctx: &Context<'_>,
        _resolver_ctx: &ResolverContext<'_>,
        last_resolver_value: Option<&ResolvedValue>,
    ) -> Result<ResolvedValue, Error> {
        let last_resolver_value = last_resolver_value.map(|val| val.data_resolved.borrow());

        let headers = ctx
            .registry()
            .http_headers
            .get(&self.api_name)
            .map(Vec::as_slice)
            .unwrap_or(&[]);

        let url = self.build_url(ctx, last_resolver_value)?;
        let mut request = dbg!(surf::RequestBuilder::new(
            self.method.parse()?,
            Url::parse(&url)?
        ));

        for (name, value) in headers {
            request = request.header(name.as_str(), value);
        }

        if let Some(request_body) = &self.request_body {
            let variable = request_body
                .variable_resolve_definition
                .resolve::<serde_json::Value>(ctx, last_resolver_value)?;

            match &request_body.content_type {
                RequestBodyContentType::Json => request = request.body_json(dbg!(&variable))?,
                RequestBodyContentType::FormEncoded(encoding_styles) => {
                    request = request.content_type(surf::http::mime::FORM);
                    request = request.body_string(
                        String::new().apply_body_parameters(encoding_styles, variable)?,
                    );
                }
            }
        }

        let mut response = request.await.map_err(|e| Error::new(e.to_string()))?;

        let data = response
            .body_json::<serde_json::Value>()
            .await
            .map_err(|e| Error::new(e.to_string()))?;

        Ok(ResolvedValue::new(Arc::new(dbg!(data))))
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
