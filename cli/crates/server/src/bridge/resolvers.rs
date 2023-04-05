use std::collections::HashMap;

use crate::types::ServerMessage;

use super::errors::ApiError;

#[derive(serde::Serialize)]
struct ResolverContext<'a> {
    env: &'a HashMap<String, String>,
}

#[derive(serde::Serialize)]
struct ResolverArgs<'a> {
    context: ResolverContext<'a>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResolverResponse {
    log_entries: Vec<String>,
    #[serde(flatten)]
    rest: serde_json::Value,
}

pub async fn invoke_resolver(
    event_bus: &tokio::sync::mpsc::Sender<ServerMessage>,
    port: u16,
    resolver_name: &str,
    payload: &serde_json::Value,
) -> Result<serde_json::Value, ApiError> {
    use futures_util::TryFutureExt;
    trace!("resolver invocation of '{resolver_name}'");
    let ResolverResponse { log_entries, rest } = reqwest::Client::new()
        .post(format!("http://127.0.0.1:{port}/resolver/{resolver_name}/invoke"))
        .json(&payload)
        .send()
        .inspect_err(|err| error!("resolver worker error: {err:?}"))
        .await
        .map_err(|_| ApiError::ServerError)?
        .json()
        .inspect_err(|err| error!("resolver worker error: {err:?}"))
        .await
        .map_err(|_| ApiError::ServerError)?;

    for message in log_entries {
        event_bus
            .send(ServerMessage::ResolverMessage {
                resolver_name: resolver_name.to_owned(),
                message,
            })
            .await
            .unwrap();
    }

    Ok(rest)
}
