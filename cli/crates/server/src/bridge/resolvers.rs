use std::collections::HashMap;

use crate::types::ServerMessage;

use common::types::ResolverMessageLevel;

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
struct ResolverMessage {
    message: String,
    level: ResolverMessageLevel,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResolverResponse {
    log_entries: Vec<ResolverMessage>,
    #[serde(flatten)]
    rest: serde_json::Value,
}

pub async fn invoke_resolver(
    bridge_sender: &tokio::sync::mpsc::Sender<ServerMessage>,
    port: u16,
    resolver_name: &str,
    payload: &serde_json::Value,
) -> Result<serde_json::Value, ApiError> {
    use futures_util::TryFutureExt;
    trace!("resolver invocation of '{resolver_name}'");
    let json_string = reqwest::Client::new()
        .post(format!("http://127.0.0.1:{port}/resolver/{resolver_name}/invoke"))
        .json(&payload)
        .send()
        .inspect_err(|err| error!("resolver worker error: {err:?}"))
        .await
        .map_err(|_| ApiError::ServerError)?
        .text()
        .inspect_err(|err| error!("resolver worker error: {err:?}"))
        .await
        .map_err(|_| ApiError::ServerError)?;

    let ResolverResponse { log_entries, rest } = serde_json::from_str(&json_string).map_err(|err| {
        error!("deserialisation from '{json_string}' failed: {err:?}");
        ApiError::ServerError
    })?;

    for ResolverMessage { level, message } in log_entries {
        bridge_sender
            .send(ServerMessage::ResolverMessage {
                resolver_name: resolver_name.to_owned(),
                level,
                message,
            })
            .await
            .unwrap();
    }

    Ok(rest)
}
