use std::{
    borrow::{Borrow, Cow},
    sync::Arc,
};

use surf::Url;

use crate::{registry::variables::VariableResolveDefinition, Context, Error};

use super::{ResolvedValue, ResolverContext};

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub struct HttpResolver {
    pub method: String,
    pub url: String,
    pub api_name: String,
    pub path_parameters: Vec<PathParameter>,
    pub query_parameters: Vec<QueryParameter>,
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

#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub enum QueryParameterEncodingStyle {
    Form,
    FormExploded,
    DeepObject,
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

        let mut url = self.url.clone();

        for param in &self.path_parameters {
            let variable = param
                .variable_resolve_definition
                .resolve(ctx, last_resolver_value)?;

            url = url.apply_path_parameter(&param, variable)?;
        }

        for param in &self.query_parameters {
            let variable = param
                .variable_resolve_definition
                .resolve(ctx, last_resolver_value)?;

            url = url.apply_query_parameter(&param, variable)?;
        }

        let mut request = surf::get(&url);

        for (name, value) in headers {
            request = request.header(name.as_str(), value);
        }

        let mut response = request.await.map_err(|e| Error::new(e.to_string()))?;

        let data = response
            .body_json::<serde_json::Value>()
            .await
            .map_err(|e| Error::new(e.to_string()))?;

        Ok(ResolvedValue::new(Arc::new(data)))
    }
}

trait ParamApply {
    fn apply_path_parameter(
        self,
        param: &PathParameter,
        variable: serde_json::Value,
    ) -> Result<String, Error>;

    fn apply_query_parameter(
        self,
        param: &QueryParameter,
        variable: serde_json::Value,
    ) -> Result<String, Error>;
}

impl ParamApply for String {
    fn apply_path_parameter(
        self,
        param: &PathParameter,
        variable: serde_json::Value,
    ) -> Result<String, Error> {
        let name = &param.name;

        Ok(self.replace(
            &format!("{{{name}}}"),
            json_to_path_string(&variable)?.borrow(),
        ))
    }

    fn apply_query_parameter(
        self,
        param: &QueryParameter,
        variable: serde_json::Value,
    ) -> Result<String, Error> {
        let name = &param.name;

        // TODO: properly handle the various other ways the spec says you can encode text into query parameters.

        let mut url = Url::parse(&self).unwrap();
        url.query_pairs_mut()
            .append_pair(name, json_to_path_string(&variable)?.borrow())
            .finish();

        Ok(url.to_string())
    }
}

fn json_to_path_string(value: &serde_json::Value) -> Result<Cow<'_, str>, Error> {
    use serde_json::Value;
    match value {
        Value::Bool(b) => Ok(Cow::Owned(b.to_string())),
        Value::Number(number) => Ok(Cow::Owned(number.to_string())),
        Value::String(string) => Ok(Cow::Borrowed(string)),
        Value::Null => Err(Error::new("HTTP path parameters cannot be null")),
        Value::Array(_) => Err(Error::new("HTTP path parameters cannot be arrays")),
        Value::Object(_) => Err(Error::new("HTTP path parameters cannot be objects")),
    }
}
