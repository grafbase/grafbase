use std::{collections::HashMap, process::Stdio};

use crate::types::ServerMessage;

use common::{environment::Environment, types::ResolverMessageLevel};
use tokio::process::Command;

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

async fn wait_until_resolver_ready(resolver_worker_port: u16, resolver_name: &str) -> Result<bool, reqwest::Error> {
    const RESOLVER_WORKER_MINIFLARE_READY_RETRY_COUNT: usize = 50;
    const RESOLVER_WORKER_MINIFLARE_READY_RETRY_INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);

    for _ in 0..RESOLVER_WORKER_MINIFLARE_READY_RETRY_COUNT {
        trace!("readiness check of resolver '{resolver_name}' under port {resolver_worker_port}");
        if is_resolver_ready(resolver_worker_port).await? {
            trace!("resolver '{resolver_name}' ready under port {resolver_worker_port}");
            return Ok(true);
        }
        tokio::time::sleep(RESOLVER_WORKER_MINIFLARE_READY_RETRY_INTERVAL).await;
    }
    Ok(false)
}

async fn is_resolver_ready(resolver_worker_port: u16) -> Result<bool, reqwest::Error> {
    match reqwest::get(format!("http://127.0.0.1:{resolver_worker_port}/health"))
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|err| {
            trace!("error: {err}");
            err
        }) {
        Ok(_) => Ok(true),
        Err(err) if err.is_connect() => Ok(false),
        Err(other) => Err(other),
    }
}

pub async fn spawn_miniflare(
    resolver_name: &str,
    main_worker_port: u16,
    package_json_path: std::path::PathBuf,
    wrangler_toml_path: std::path::PathBuf,
    tracing: bool,
) -> Result<(tokio::task::JoinHandle<()>, u16), ApiError> {
    let environment = Environment::get();

    let resolver_worker_port = crate::servers::find_available_port_for_internal_use(main_worker_port)
        .map_err(|_| ApiError::CouldNotFindPortForResolverWorker)?;
    let resolver_worker_port_string = resolver_worker_port.to_string();

    let miniflare_path = environment
        .user_dot_grafbase_path
        .join(crate::consts::MINIFLARE_CLI_JS_PATH)
        .canonicalize()
        .unwrap();

    let resolver_name_cloned = resolver_name.to_owned();
    let join_handle = tokio::spawn(async move {
        let miniflare_arguments = &[
            // used by miniflare when running normally as well
            "--experimental-vm-modules",
            miniflare_path.to_str().unwrap(),
            "--modules",
            "--debug",
            "--host",
            "127.0.0.1",
            "--port",
            &resolver_worker_port_string,
            "--package",
            package_json_path.to_str().unwrap(),
            "--no-update-check",
            "--no-cf-fetch",
            "--wrangler-config",
            wrangler_toml_path.to_str().unwrap(),
        ];
        let miniflare_command = miniflare_arguments.join(" ");

        let mut miniflare = Command::new("node");
        miniflare
            // Unbounded worker limit
            .env("MINIFLARE_SUBREQUEST_LIMIT", "1000")
            .args(miniflare_arguments)
            .stdout(if tracing { Stdio::inherit() } else { Stdio::piped() })
            .stderr(if tracing { Stdio::inherit() } else { Stdio::piped() })
            .current_dir(wrangler_toml_path.parent().unwrap())
            .kill_on_drop(true);
        trace!("Spawning resolver '{resolver_name_cloned}': {miniflare_command}");
        let miniflare = miniflare.spawn().unwrap();
        let output = miniflare.wait_with_output().await.unwrap();

        assert!(
            output.status.success(),
            "resolver worker failed: '{}'",
            String::from_utf8_lossy(&output.stderr).into_owned()
        );
    });

    if wait_until_resolver_ready(resolver_worker_port, resolver_name)
        .await
        .map_err(|_| ApiError::ResolverSpawnError)?
    {
        Ok((join_handle, resolver_worker_port))
    } else {
        Err(ApiError::ResolverSpawnError)
    }
}

pub async fn invoke_resolver(
    bridge_sender: &tokio::sync::mpsc::Sender<ServerMessage>,
    resolver_worker_port: u16,
    resolver_name: &str,
    payload: &serde_json::Value,
) -> Result<serde_json::Value, ApiError> {
    use futures_util::TryFutureExt;
    trace!("resolver invocation of '{resolver_name}'");
    let json_string = reqwest::Client::new()
        .post(format!("http://127.0.0.1:{resolver_worker_port}/invoke"))
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
