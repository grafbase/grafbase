use std::{
    borrow::{Borrow, Cow},
    sync::Arc,
};

use crate::{registry::variables::VariableResolveDefinition, Context, Error};

use super::{ResolvedValue, ResolverContext};

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub struct HttpResolver {
    pub method: String,
    pub url: String,
    pub api_name: String,
    pub path_parameters: Vec<Parameter>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub struct Parameter {
    pub name: String,
    pub variable_resolve_definition: VariableResolveDefinition,
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
            url = param.apply_as_path_parameter(&url, ctx, last_resolver_value)?;
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

impl Parameter {
    fn apply_as_path_parameter(
        &self,
        url: &str,
        ctx: &Context<'_>,
        last_resolver_value: Option<&serde_json::Value>,
    ) -> Result<String, Error> {
        let name = &self.name;
        let variable = self
            .variable_resolve_definition
            .resolve::<serde_json::Value>(ctx, last_resolver_value)?;

        Ok(url.replace(&format!("{{{name}}}"), json_to_string(&variable)?.borrow()))
    }
}

fn json_to_string(value: &serde_json::Value) -> Result<Cow<'_, str>, Error> {
    use serde_json::Value;
    match value {
        Value::Bool(b) => Ok(Cow::Owned(b.to_string())),
        Value::Number(number) => Ok(Cow::Owned(number.to_string())),
        Value::String(string) => Ok(Cow::Borrowed(string)),
        Value::Null => Err(Error::new("HTTP URL parameters cannot be null")),
        Value::Array(_) => Err(Error::new("HTTP URL parameters cannot be arrays")),
        Value::Object(_) => Err(Error::new("HTTP URL parameters cannot be objects")),
    }
}
