use std::sync::Arc;

use crate::{Context, Error};

use super::{ResolvedValue, ResolverContext};

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub struct HttpResolver {
    pub method: String,
    pub url: String,
    pub api_name: String,
}

impl HttpResolver {
    pub async fn resolve(
        &self,
        ctx: &Context<'_>,
        _resolver_ctx: &ResolverContext<'_>,
        _last_resolver_value: Option<&ResolvedValue>,
    ) -> Result<ResolvedValue, Error> {
        let headers = ctx
            .registry()
            .http_headers
            .get(&self.api_name)
            .map(Vec::as_slice)
            .unwrap_or(&[]);

        let mut request = surf::get(&self.url);

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
