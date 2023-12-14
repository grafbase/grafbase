use super::udf::UdfRuntime;
use crate::bridge::log::log_event_endpoint;
use crate::bridge::udf::invoke_udf_endpoint;
use crate::config::DetectedUdf;
use crate::errors::ServerError;
use crate::types::MessageSender;
use axum::{routing::post, Router};
use common::environment::Project;

use tokio::fs;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use std::future::IntoFuture;
use std::net::TcpListener;
use std::num::NonZeroUsize;
use std::sync::Arc;

use tower_http::trace::TraceLayer;

pub struct HandlerState {
    pub message_sender: MessageSender,
    pub udf_runtime: UdfRuntime,
    pub tracing: bool,
    pub registry: Arc<engine::Registry>,
}

// Not great, but I don't want to expose HandlerState and nor do I want to change everything now...
#[async_trait::async_trait]
pub trait BridgeState: Send + Sync {
    async fn build_all_udfs(&self, udfs: Vec<DetectedUdf>, parallelism: NonZeroUsize) -> Result<(), ServerError>;
}

#[async_trait::async_trait]
impl BridgeState for Arc<HandlerState> {
    async fn build_all_udfs(&self, udfs: Vec<DetectedUdf>, parallelism: NonZeroUsize) -> Result<(), ServerError> {
        Ok(self.udf_runtime.build_all(udfs, parallelism).await?)
    }
}

pub async fn build_router(
    message_sender: MessageSender,
    registry: Arc<engine::Registry>,
    tracing: bool,
) -> Result<(Router, Arc<HandlerState>), ServerError> {
    let project = Project::get();

    let environment_variables: std::collections::HashMap<_, _> = crate::environment::variables().collect();

    match project.database_directory_path.try_exists() {
        Ok(true) => {}
        Ok(false) => fs::create_dir_all(&project.database_directory_path)
            .await
            .map_err(ServerError::CreateDatabaseDir)?,
        Err(error) => return Err(ServerError::ReadDatabaseDir(error)),
    }

    let udf_runtime = UdfRuntime::new(environment_variables, registry.clone(), tracing, message_sender.clone());
    let handler_state = Arc::new(HandlerState {
        message_sender,
        udf_runtime,
        tracing,
        registry,
    });

    let router = Router::new()
        .route("/invoke-udf", post(invoke_udf_endpoint))
        .route("/log-event", post(log_event_endpoint))
        .with_state(handler_state.clone())
        .layer(TraceLayer::new_for_http());

    Ok((router, handler_state))
}

pub async fn spawn(
    tcp_listener: TcpListener,
    message_sender: MessageSender,
    cancel_token: CancellationToken,
    registry: Arc<engine::Registry>,
    tracing: bool,
) -> Result<JoinHandle<Result<(), hyper::Error>>, ServerError> {
    let (router, ..) = build_router(message_sender, registry, tracing).await?;

    let server = axum::Server::from_tcp(tcp_listener)?
        .serve(router.into_make_service())
        .with_graceful_shutdown(cancel_token.cancelled_owned());

    Ok(tokio::spawn(server.into_future()))
}
