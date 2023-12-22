use std::borrow::Cow;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;

use crate::config::DetectedUdf;
use crate::errors::UdfBuildError;
use crate::types::{MessageSender, ServerMessage};
use crate::udf_builder::udf_url_path;

use axum::extract::State;
use axum::Json;
use common::types::UdfKind;
use common::{environment::Environment, types::LogLevel};
use futures_util::{pin_mut, TryFutureExt, TryStreamExt};
use tokio::process::Command;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::{Mutex, Notify};

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

enum UdfWorkerStatus {
    BuildInProgress {
        notify: Arc<Notify>,
    },
    Available {
        #[allow(dead_code)]
        miniflare_handle: Arc<tokio::task::JoinHandle<()>>,
        worker_port: u16,
    },
    BuildFailed,
}

struct UdfWorker {
    name: String,
    directory: PathBuf,
}

pub struct UdfRuntime {
    udf_workers: Mutex<HashMap<(String, UdfKind), UdfWorkerStatus>>,
    environment_variables: HashMap<String, String>,
    registry: Arc<engine::Registry>,
    tracing: bool,
    message_sender: MessageSender,
}

#[allow(clippy::too_many_lines)]
pub async fn invoke_udf_endpoint(
    State(handler_state): State<Arc<HandlerState>>,
    Json(payload): Json<UdfInvocation>,
) -> Result<Json<serde_json::Value>, ApiError> {
    trace!("UDF invocation\n\n{:#?}\n", payload);

    let UdfInvocation {
        request_id,
        name: udf_name,
        payload,
        udf_kind,
    } = payload;

    let udf_worker_port = handler_state.udf_runtime.build_udf(udf_name.clone(), udf_kind).await?;

    invoke(
        &handler_state.message_sender,
        &request_id,
        udf_worker_port,
        udf_kind,
        &udf_name,
        &payload,
    )
    .await
    .map(Json)
}

impl UdfRuntime {
    pub fn new(
        environment_variables: HashMap<String, String>,
        registry: Arc<engine::Registry>,
        tracing: bool,
        message_sender: MessageSender,
    ) -> Self {
        Self {
            udf_workers: Mutex::default(),
            environment_variables,
            registry,
            tracing,
            message_sender,
        }
    }

    pub async fn build_all(&self, udfs: Vec<DetectedUdf>, parallelism: NonZeroUsize) -> Result<(), UdfBuildError> {
        let start = std::time::Instant::now();
        self.message_sender
            .send(ServerMessage::StartUdfBuildAll)
            .expect("receiver is not never closed");
        let udf_workers = self.build_all_udf_workers(udfs.clone(), parallelism).await?;
        self.message_sender
            .send(ServerMessage::CompleteUdfBuildAll {
                duration: start.elapsed(),
            })
            .expect("receiver is not never closed");
        let (join_handle, port) = self.spawn_multi_worker_miniflare(udf_workers).await?;
        let join_handle = Arc::new(join_handle);
        let mut builds = self.udf_workers.lock().await;
        for udf in udfs {
            builds.insert(
                (udf.udf_name, udf.udf_kind),
                UdfWorkerStatus::Available {
                    miniflare_handle: join_handle.clone(),
                    worker_port: port,
                },
            );
        }
        Ok(())
    }
}

impl UdfRuntime {
    async fn spawn_multi_worker_miniflare(
        &self,
        udf_workers: Vec<UdfWorker>,
    ) -> Result<(tokio::task::JoinHandle<()>, u16), UdfBuildError> {
        let mut miniflare_arguments: Vec<_> = [
            // used by miniflare when running normally as well
            "--experimental-vm-modules",
            "./node_modules/miniflare/dist/src/cli.js",
            "--modules",
            "--host",
            "127.0.0.1",
            "--port",
            "0",
            "--no-update-check",
            "--no-cf-fetch",
            "--do-persist",
        ]
        .into_iter()
        .map(Cow::Borrowed)
        .collect();
        if self.tracing {
            miniflare_arguments.push("--debug".into());
        }

        miniflare_arguments.extend(udf_workers.into_iter().flat_map(|UdfWorker { name, directory }| {
            [
                Cow::Borrowed("--mount"),
                format!("{name}={path}", name = slug::slugify(name), path = directory.display()).into(),
            ]
        }));

        let environment = Environment::get();
        let mut miniflare = Command::new("node");
        miniflare
            // Unbounded worker limit
            .env("MINIFLARE_SUBREQUEST_LIMIT", "1000")
            .args(miniflare_arguments.iter().map(std::convert::AsRef::as_ref))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(&environment.user_dot_grafbase_path)
            .kill_on_drop(true);
        trace!("Spawning {miniflare:?}");
        let mut miniflare = miniflare.spawn().unwrap();

        let bound_port = {
            use tokio::io::AsyncBufReadExt;
            use tokio_stream::wrappers::LinesStream;
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
        let join_handle = tokio::spawn(async move {
            let outcome = miniflare.wait_with_output().await.unwrap();
            assert!(
                outcome.status.success(),
                "Miniflare failed: '{}'",
                String::from_utf8_lossy(&outcome.stderr).into_owned()
            );
        });

        Ok((join_handle, bound_port))
    }

    async fn build_all_udf_workers(
        &self,
        udfs: impl IntoIterator<Item = DetectedUdf>,
        parallelism: NonZeroUsize,
    ) -> Result<Vec<UdfWorker>, UdfBuildError> {
        use futures_util::StreamExt;

        let environment = Environment::get();

        let mut resolvers_iterator = udfs.into_iter().peekable();
        if resolvers_iterator.peek().is_none() {
            return Ok(vec![]);
        }

        futures_util::stream::iter(resolvers_iterator)
            .map(Ok)
            .map_ok(|DetectedUdf { udf_name, udf_kind, .. }| async move {
                match crate::udf_builder::build(
                    environment,
                    &self.environment_variables,
                    udf_kind,
                    &udf_name,
                    self.tracing,
                    self.registry.enable_kv,
                )
                .await
                {
                    Ok((_, wrangler_toml_path)) => Ok(UdfWorker {
                        name: udf_name,
                        directory: wrangler_toml_path.parent().unwrap().to_owned(),
                    }),
                    Err(err) => {
                        self.message_sender
                            .send(ServerMessage::CompilationError(format!(
                                "{udf_kind} '{udf_name}' failed to build: {err}"
                            )))
                            .expect("receiver is not never closed");

                        Err(err)
                    }
                }
            })
            .try_buffer_unordered(parallelism.into())
            .try_collect()
            .await
    }

    async fn build_udf(&self, udf_name: String, udf_kind: UdfKind) -> Result<u16, ApiError> {
        let environment = Environment::get();
        let udf_worker_port = loop {
            let notify = {
                let mut udf_builds: tokio::sync::MutexGuard<'_, _> = self.udf_workers.lock().await;

                if let Some(udf_build) = udf_builds.get(&(udf_name.clone(), udf_kind)) {
                    match udf_build {
                        UdfWorkerStatus::Available { worker_port, .. } => break *worker_port,
                        UdfWorkerStatus::BuildFailed => return Err(ApiError::UdfInvocation),
                        UdfWorkerStatus::BuildInProgress { notify } => {
                            // If the resolver build happening within another invocation has been cancelled
                            // due to the invocation having been interrupted by the HTTP client, start a new build.
                            if Arc::strong_count(&notify) == 1 {
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
                        UdfWorkerStatus::BuildInProgress { notify: notify.clone() },
                    );
                    notify
                }
            };

            let start = std::time::Instant::now();
            self.message_sender
                .send(ServerMessage::StartUdfBuild {
                    udf_kind,
                    udf_name: udf_name.clone(),
                })
                .unwrap();

            match crate::udf_builder::build(
                environment,
                &self.environment_variables,
                udf_kind,
                &udf_name,
                self.tracing,
                self.registry.enable_kv,
            )
            .and_then(|(package_json_path, wrangler_toml_path)| {
                super::udf::spawn_miniflare(udf_kind, &udf_name, package_json_path, wrangler_toml_path, self.tracing)
            })
            .await
            {
                Ok((miniflare_handle, worker_port)) => {
                    self.udf_workers.lock().await.insert(
                        (udf_name.clone(), udf_kind),
                        UdfWorkerStatus::Available {
                            miniflare_handle: Arc::new(miniflare_handle),
                            worker_port,
                        },
                    );
                    notify.notify_waiters();

                    self.message_sender
                        .send(ServerMessage::CompleteUdfBuild {
                            udf_kind,
                            udf_name: udf_name.clone(),
                            duration: start.elapsed(),
                        })
                        .unwrap();

                    break worker_port;
                }
                Err(err) => {
                    error!("Build of {udf_kind} '{udf_name}' failed: {err:?}");
                    self.message_sender
                        .send(ServerMessage::CompilationError(format!(
                            "{udf_kind} '{udf_name}' failed to build: {err}"
                        )))
                        .unwrap();
                }
            };

            self.udf_workers
                .lock()
                .await
                .insert((udf_name.clone(), udf_kind), UdfWorkerStatus::BuildFailed);
            notify.notify_waiters();
            return Err(ApiError::UdfInvocation);
        };
        Ok(udf_worker_port)
    }
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
    let result = reqwest::get(format!("http://127.0.0.1:{resolver_worker_port}/health"))
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|err| {
            trace!("error: {err}");
            err
        });

    match result {
        Ok(_) => Ok(true),
        Err(err) if err.is_connect() => Ok(false),
        Err(other) => Err(other),
    }
}

async fn spawn_miniflare(
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

async fn invoke(
    bridge_sender: &UnboundedSender<ServerMessage>,
    request_id: &str,
    udf_worker_port: u16,
    udf_kind: UdfKind,
    udf_name: &str,
    payload: &serde_json::Value,
) -> Result<serde_json::Value, ApiError> {
    let url = format!("http://127.0.0.1:{udf_worker_port}{}", udf_url_path(udf_kind, udf_name));
    trace!("Invocation of {udf_kind} '{udf_name}' as {url} with payload {payload}");

    let json_string = reqwest::Client::new()
        .post(url)
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
        bridge_sender.send(message).expect("receiver is not never closed");
    }

    Ok(value)
}
