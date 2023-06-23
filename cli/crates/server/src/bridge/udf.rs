use std::process::Stdio;

use crate::types::ServerMessage;

use common::types::UdfKind;
use common::{environment::Environment, types::UdfMessageLevel};
use futures_util::{pin_mut, TryStreamExt};
use tokio::process::Command;

use super::errors::ApiError;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct UdfMessage {
    message: String,
    level: UdfMessageLevel,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct UdfResponse {
    log_entries: Vec<UdfMessage>,
    #[serde(flatten)]
    rest: serde_json::Value,
}

async fn wait_until_udf_ready(worker_port: u16, udf_kind: UdfKind, udf_name: &str) -> Result<bool, reqwest::Error> {
    const RESOLVER_WORKER_MINIFLARE_READY_RETRY_COUNT: usize = 50;
    const RESOLVER_WORKER_MINIFLARE_READY_RETRY_INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);

    for _ in 0..RESOLVER_WORKER_MINIFLARE_READY_RETRY_COUNT {
        trace!("readiness check of {udf_kind} '{udf_name}' under port {worker_port}");
        if is_udf_ready(worker_port).await? {
            trace!("{udf_kind} '{udf_name}' ready under port {worker_port}");
            return Ok(true);
        }
        tokio::time::sleep(RESOLVER_WORKER_MINIFLARE_READY_RETRY_INTERVAL).await;
    }
    Ok(false)
}

async fn is_udf_ready(resolver_worker_port: u16) -> Result<bool, reqwest::Error> {
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
    udf_kind: UdfKind,
    udf_name: &str,
    package_json_path: std::path::PathBuf,
    wrangler_toml_path: std::path::PathBuf,
    _tracing: bool,
) -> Result<(tokio::task::JoinHandle<()>, u16), ApiError> {
    use tokio::io::AsyncBufReadExt;
    use tokio_stream::wrappers::LinesStream;

    let environment = Environment::get();

    let miniflare_path = environment
        .user_dot_grafbase_path
        .join(crate::consts::MINIFLARE_CLI_JS_PATH)
        .canonicalize()
        .unwrap();

    let (join_handle, resolver_worker_port) = {
        let miniflare_arguments = &[
            // used by miniflare when running normally as well
            "--experimental-vm-modules",
            miniflare_path.to_str().unwrap(),
            "--modules",
            "--debug",
            "--host",
            "127.0.0.1",
            "--port",
            "0",
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
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(wrangler_toml_path.parent().unwrap())
            .kill_on_drop(true);
        trace!("Spawning {udf_kind} '{udf_name}': {miniflare_command}");

        let mut miniflare = miniflare.spawn().unwrap();
        let bound_port = {
            let stdout = miniflare.stdout.as_mut().unwrap();
            let lines_stream = LinesStream::new(tokio::io::BufReader::new(stdout).lines())
                .inspect_ok(|line| trace!("miniflare: {line}"));

            let filtered_lines_stream = lines_stream.try_filter_map(|line| {
                futures_util::future::ready(Ok(line
                    .split("Listening on")
                    .skip(1)
                    .flat_map(|bound_address| bound_address.split(':'))
                    .nth(1)
                    .and_then(|value| value.trim().parse::<u16>().ok())))
            });
            pin_mut!(filtered_lines_stream);
            filtered_lines_stream
                .try_next()
                .await
                .ok()
                .flatten()
                .expect("must be present")
        };
        trace!("Bound to port: {bound_port}");
        let udf_name = udf_name.to_owned();
        let join_handle = tokio::spawn(async move {
            let outcome = miniflare.wait_with_output().await.unwrap();
            assert!(
                outcome.status.success(),
                "udf worker {udf_kind} '{udf_name}' failed: '{}'",
                String::from_utf8_lossy(&outcome.stderr).into_owned()
            );
        });

        (join_handle, bound_port)
    };

    if wait_until_udf_ready(resolver_worker_port, udf_kind, udf_name)
        .await
        .map_err(|_| ApiError::UdfSpawnError)?
    {
        Ok((join_handle, resolver_worker_port))
    } else {
        Err(ApiError::UdfSpawnError)
    }
}

pub async fn invoke(
    bridge_sender: &tokio::sync::mpsc::Sender<ServerMessage>,
    udf_worker_port: u16,
    udf_kind: UdfKind,
    udf_name: &str,
    payload: &serde_json::Value,
) -> Result<serde_json::Value, ApiError> {
    use futures_util::TryFutureExt;
    trace!("Invocation of {udf_kind} '{udf_name}'");
    let json_string = reqwest::Client::new()
        .post(format!("http://127.0.0.1:{udf_worker_port}/invoke"))
        .json(&payload)
        .send()
        .inspect_err(|err| error!("{udf_kind} '{udf_name}' worker error: {err:?}"))
        .await
        .map_err(|_| ApiError::UdfInvocation)?
        .text()
        .inspect_err(|err| error!("{udf_kind} '{udf_name}' worker error: {err:?}"))
        .await
        .map_err(|_| ApiError::UdfInvocation)?;

    let UdfResponse { log_entries, rest } = serde_json::from_str(&json_string).map_err(|err| {
        error!("deserialization from '{json_string}' failed: {err:?}");
        ApiError::UdfInvocation
    })?;

    for UdfMessage { level, message } in log_entries {
        bridge_sender
            .send(ServerMessage::UdfMessage {
                udf_kind,
                udf_name: udf_name.to_owned(),
                level,
                message,
            })
            .await
            .unwrap();
    }

    Ok(rest)
}
