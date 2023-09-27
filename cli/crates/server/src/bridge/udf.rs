use std::process::Stdio;
use std::sync::Arc;

use crate::errors::UdfBuildError;
use crate::types::ServerMessage;

use axum::extract::State;
use axum::Json;
use common::types::UdfKind;
use common::{environment::Environment, types::LogLevel};
use futures_util::{pin_mut, TryFutureExt, TryStreamExt};
use tokio::process::Command;
use tokio::sync::Notify;

use super::errors::ApiError;
use super::server::HandlerState;
use super::types::UdfInvocation;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct UdfMessage {
    logged_at: u64,
    message: String,
    level: LogLevel,
}

#[serde_with::serde_as]
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct FetchRequest {
    logged_at: u64,
    url: String,
    #[serde_as(as = "serde_with::DurationMilliSeconds<u64>")]
    duration: std::time::Duration,
    method: String,
    status_code: u16,
    body: Option<String>,
    content_type: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct UdfResponse {
    fetch_requests: Vec<FetchRequest>,
    log_entries: Vec<UdfMessage>,
    value: serde_json::Value,
}

pub enum UdfBuild {
    InProgress {
        notify: Arc<Notify>,
    },
    Succeeded {
        #[allow(dead_code)]
        miniflare_handle: tokio::task::JoinHandle<()>,
        worker_port: u16,
    },
    Failed,
}

#[allow(clippy::too_many_lines)]
pub async fn invoke_udf_endpoint(
    State(handler_state): State<Arc<HandlerState>>,
    Json(payload): Json<UdfInvocation>,
) -> Result<Json<serde_json::Value>, ApiError> {
    trace!("UDF invocation\n\n{:#?}\n", payload);

    let environment = Environment::get();
    let UdfInvocation {
        request_id,
        name: udf_name,
        payload,
        udf_kind,
    } = payload;

    let udf_worker_port = loop {
        let notify = {
            let mut udf_builds: tokio::sync::MutexGuard<'_, _> = handler_state.udf_builds.lock().await;

            if let Some(udf_build) = udf_builds.get(&(udf_name.clone(), udf_kind)) {
                match udf_build {
                    UdfBuild::Succeeded { worker_port, .. } => break *worker_port,
                    UdfBuild::Failed => return Err(ApiError::UdfInvocation),
                    UdfBuild::InProgress { notify } => {
                        // If the resolver build happening within another invocation has been cancelled
                        // due to the invocation having been interrupted by the HTTP client, start a new build.
                        if Arc::strong_count(notify) == 1 {
                            notify.clone()
                        } else {
                            let notify = notify.clone();
                            drop(udf_builds);
                            notify.notified().await;
                            continue;
                        }
                    }
                }
            } else {
                let notify = Arc::new(Notify::new());
                udf_builds.insert(
                    (udf_name.clone(), udf_kind),
                    UdfBuild::InProgress { notify: notify.clone() },
                );
                notify
            }
        };

        let start = std::time::Instant::now();
        handler_state
            .bridge_sender
            .send(ServerMessage::StartUdfBuild {
                udf_kind,
                udf_name: udf_name.clone(),
            })
            .await
            .unwrap();

        let tracing = handler_state.tracing;
        let enable_kv = handler_state.registry.enable_kv;

        match crate::udf_builder::build(
            environment,
            &handler_state.environment_variables,
            udf_kind,
            &udf_name,
            tracing,
            enable_kv,
        )
        .and_then(|(package_json_path, wrangler_toml_path)| {
            super::udf::spawn_miniflare(udf_kind, &udf_name, package_json_path, wrangler_toml_path, tracing)
        })
        .await
        {
            Ok((miniflare_handle, worker_port)) => {
                handler_state.udf_builds.lock().await.insert(
                    (udf_name.clone(), udf_kind),
                    UdfBuild::Succeeded {
                        miniflare_handle,
                        worker_port,
                    },
                );
                notify.notify_waiters();

                handler_state
                    .bridge_sender
                    .send(ServerMessage::CompleteUdfBuild {
                        udf_kind,
                        udf_name: udf_name.clone(),
                        duration: start.elapsed(),
                    })
                    .await
                    .unwrap();

                break worker_port;
            }
            Err(err) => {
                error!("Build of {udf_kind} '{udf_name}' failed: {err:?}");
                handler_state
                    .bridge_sender
                    .send(ServerMessage::CompilationError(format!(
                        "{udf_kind} '{udf_name}' failed to build: {err}"
                    )))
                    .await
                    .unwrap();
            }
        };

        handler_state
            .udf_builds
            .lock()
            .await
            .insert((udf_name.clone(), udf_kind), UdfBuild::Failed);
        notify.notify_waiters();
        return Err(ApiError::UdfInvocation);
    };

    super::udf::invoke(
        &handler_state.bridge_sender,
        &request_id,
        udf_worker_port,
        udf_kind,
        &udf_name,
        &payload,
    )
    .await
    .map(Json)
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
    tracing: bool,
) -> Result<(tokio::task::JoinHandle<()>, u16), UdfBuildError> {
    use tokio::io::AsyncBufReadExt;
    use tokio_stream::wrappers::LinesStream;

    let environment = Environment::get();

    let miniflare_path = environment
        .user_dot_grafbase_path
        .join(crate::consts::MINIFLARE_CLI_JS_PATH)
        .canonicalize()
        .unwrap();

    let (join_handle, resolver_worker_port) = {
        let mut miniflare_arguments = vec![
            // used by miniflare when running normally as well
            "--experimental-vm-modules",
            miniflare_path.to_str().unwrap(),
            "--modules",
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
        if tracing {
            miniflare_arguments.push("--debug");
        }
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
            let mut lines_skipped_over = vec![];
            let filtered_lines_stream =
                LinesStream::new(tokio::io::BufReader::new(stdout).lines()).try_filter_map(|line: String| {
                    trace!("miniflare: {line}");
                    let port = line
                        .split("Listening on")
                        .skip(1)
                        .flat_map(|bound_address| bound_address.split(':'))
                        .nth(1)
                        .and_then(|value| value.trim().parse::<u16>().ok());
                    lines_skipped_over.push(line);
                    futures_util::future::ready(Ok(port))
                });
            pin_mut!(filtered_lines_stream);
            filtered_lines_stream.try_next().await.ok().flatten().ok_or_else(|| {
                UdfBuildError::MiniflareSpawnFailedWithOutput {
                    output: lines_skipped_over.join("\n"),
                }
            })?
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
        .map_err(|_| UdfBuildError::MiniflareSpawnFailed)?
    {
        Ok((join_handle, resolver_worker_port))
    } else {
        Err(UdfBuildError::MiniflareSpawnFailed)
    }
}

pub async fn invoke(
    bridge_sender: &tokio::sync::mpsc::Sender<ServerMessage>,
    request_id: &str,
    udf_worker_port: u16,
    udf_kind: UdfKind,
    udf_name: &str,
    payload: &serde_json::Value,
) -> Result<serde_json::Value, ApiError> {
    trace!("Invocation of {udf_kind} '{udf_name}' with payload {payload}");
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

    let UdfResponse {
        fetch_requests,
        log_entries,
        value,
    } = serde_json::from_str(&json_string).map_err(|err| {
        error!("deserialization from '{json_string}' failed: {err:?}");
        ApiError::UdfInvocation
    })?;

    let mut messages = vec![];

    for UdfMessage {
        logged_at: logged_time,
        level,
        message,
    } in log_entries
    {
        messages.push((
            logged_time,
            ServerMessage::RequestScopedMessage {
                request_id: request_id.to_owned(),
                event_type: crate::types::LogEventType::NestedEvent(
                    crate::types::NestedRequestScopedMessage::UdfMessage {
                        udf_kind,
                        udf_name: udf_name.to_owned(),
                        level,
                        message,
                    },
                ),
            },
        ));
    }

    for FetchRequest {
        logged_at: logged_time,
        url,
        duration,
        method,
        status_code,
        body,
        content_type,
    } in fetch_requests
    {
        messages.push((
            logged_time,
            ServerMessage::RequestScopedMessage {
                request_id: request_id.to_owned(),
                event_type: crate::types::LogEventType::NestedEvent(
                    crate::types::NestedRequestScopedMessage::NestedRequest {
                        url,
                        duration,
                        method,
                        status_code,
                        body,
                        content_type,
                    },
                ),
            },
        ));
    }

    messages.sort_by_key(|(logged_time, _)| *logged_time);
    for (_, message) in messages {
        bridge_sender.send(message).await.unwrap();
    }

    Ok(value)
}
