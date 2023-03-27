use std::collections::HashMap;

use super::errors::ApiError;

#[derive(serde::Serialize)]
struct ResolverContext<'a> {
    env: &'a HashMap<String, String>,
}

#[derive(serde::Serialize)]
struct ResolverArgs<'a> {
    context: ResolverContext<'a>,
}

pub async fn invoke_resolver(port: u16, resolver_name: &str) -> Result<serde_json::Value, ApiError> {
    use futures_util::TryFutureExt;
    trace!("resolver invocation of '{resolver_name}'");
    reqwest::Client::new()
        .post(format!("http://127.0.0.1:{port}/resolver/{resolver_name}/invoke"))
        .send()
        .inspect_err(|err| error!("resolver worker error: {err:?}"))
        .await
        .map_err(|_| ApiError::ServerError)?
        .json()
        .inspect_err(|err| error!("resolver worker error: {err:?}"))
        .await
        .map_err(|_| ApiError::ServerError)
}
