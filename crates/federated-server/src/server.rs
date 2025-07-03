mod cors;
mod engine_reloader;
mod events;
mod gateway;
mod graph_fetch_method;
mod graph_updater;
pub mod router;
mod state;
mod trusted_documents_client;

use self::events::UpdateEvent;
pub(crate) use gateway::CreateExtensionCatalogError;
use gateway::create_extension_catalog::create_extension_catalog;
pub use graph_fetch_method::GraphFetchMethod;
use router::RouterConfig;
pub use state::ServerState;
use ulid::Ulid;

use axum::Router;
use engine_axum::middleware::TelemetryLayer;
use engine_reloader::{EngineReloaderConfig, GatewayEngineReloader};
use gateway_config::{Config, TlsConfig};
use std::{net::SocketAddr, path::PathBuf};
use tokio::{
    signal,
    sync::{mpsc, watch},
};
use wasi_component_loader::extension::WasmHooks;

/// Start parameter for the gateway.
pub struct ServeConfig {
    pub listen_address: SocketAddr,
    /// The gateway configuration.
    pub config_receiver: watch::Receiver<Config>,
    /// The config file path for hot reload.
    pub config_path: Option<PathBuf>,
    /// If true, watches changes to the config
    /// and reloads _some_ of the things.
    pub config_hot_reload: bool,
    /// The way of loading the graph for the gateway.
    pub fetch_method: GraphFetchMethod,
    pub grafbase_access_token: Option<AccessToken>,
    pub logging_filter: String,
}

#[derive(Clone)]
pub struct AccessToken(pub String);

impl std::fmt::Debug for AccessToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("AccessToken").field(&"<REDACTED>").finish()
    }
}

/// Trait for server runtime.
#[allow(unused_variables)]
pub trait ServerRuntime: Send + Sync + 'static + Clone {
    /// Called after each request
    fn after_request(&self) {}
    /// Called when the server is ready and listening
    fn on_ready(&self, url: String) {}
    fn base_router<S>(&self) -> Option<axum::Router<S>> {
        None
    }
}

impl ServerRuntime for () {}

/// Starts the server and listens for incoming requests.
pub async fn serve(
    ServeConfig {
        listen_address,
        config_receiver,
        config_path,
        config_hot_reload,
        fetch_method,
        grafbase_access_token,
        logging_filter,
    }: ServeConfig,
    server_runtime: impl ServerRuntime,
) -> crate::Result<()> {
    let config = config_receiver.borrow().clone();
    let path = config.graph.path.clone();

    #[allow(unused)]
    let tls = config.tls.clone();

    // Create the central channel for all update events
    let (update_sender, update_receiver) = mpsc::channel::<UpdateEvent>(16);

    // Start the graph producer
    fetch_method.start_producer(update_sender.clone()).await?;

    // Bridge config updates to the central channel if hot reload is enabled
    if config_hot_reload {
        spawn_config_reloader(config_receiver, update_sender);
    }

    // We separate the hooks extension, which runs outside of the engine in the axum layers.
    let (extension_catalog, hooks_extension) = create_extension_catalog(&config).await?;

    // The engine reloads itself when the graph, or configuration changes.
    let update_handler = GatewayEngineReloader::spawn(EngineReloaderConfig {
        update_receiver,
        initial_config: config.clone(),
        extension_catalog: &extension_catalog,
        logging_filter: logging_filter.clone(),
        hot_reload_config_path: config_hot_reload.then_some(config_path).flatten(),
        access_token: grafbase_access_token,
    })
    .await?;

    let mcp_url = config
        .mcp
        .as_ref()
        .filter(|m| m.enabled)
        .map(|m| format!("http://{listen_address}{}", m.path));

    // On-request, on-response hooks extension.
    let hooks = WasmHooks::new(&config, hooks_extension, logging_filter)
        .await
        .map_err(|e| crate::Error::InternalError(e.to_string()))?;

    let inject_layers_before_cors = |router: axum::Router| {
        // Currently we're doing those after CORS handling in the request as we don't care
        // about pre-flight requests.
        let telemetry_layer = TelemetryLayer::new(
            grafbase_telemetry::metrics::meter_from_global_provider(),
            Some(listen_address),
        );

        router.layer(telemetry_layer)
    };

    let router_config = RouterConfig {
        config,
        engine: update_handler.engine_watcher(),
        server_runtime: server_runtime.clone(),
        hooks,
        inject_layers_before_cors,
    };

    // Generate all routes for the HTTP server.
    let (router, cancellation_token) = router::create(router_config).await?;

    // Finally start the HTTP server, different binding mechanism for lambda.
    cfg_if::cfg_if! {
        if #[cfg(feature = "lambda")] {
            let result = lambda_bind(&path, router, mcp_url).await;
        } else {
            let result = bind(listen_address, &path, router, tls.as_ref(), server_runtime, mcp_url).await;
        }
    }

    // The cancellation token is to stop the MCP server gracefully. It only exists
    // if the MCP server is enabled and running.
    if let Some(token) = cancellation_token {
        token.cancel();
    }

    result
}

fn spawn_config_reloader(mut config_receiver: watch::Receiver<Config>, update_sender: mpsc::Sender<UpdateEvent>) {
    tokio::spawn(async move {
        // drop the initial value
        config_receiver.changed().await.ok();

        while let Ok(()) = config_receiver.changed().await {
            let new_config = config_receiver.borrow().clone();

            if update_sender.send(UpdateEvent::config(new_config)).await.is_err() {
                break; // channel closed
            }
        }
    });
}

#[cfg_attr(feature = "lambda", allow(unused))]
async fn bind(
    addr: SocketAddr,
    path: &str,
    router: Router<()>,
    tls: Option<&TlsConfig>,
    server_runtime: impl ServerRuntime,
    mcp_url: Option<String>,
) -> crate::Result<()> {
    let app = router.into_make_service();

    let handle = axum_server::Handle::new();

    // Spawn a task to gracefully shutdown server.
    tokio::spawn(graceful_shutdown(handle.clone()));

    let handle_for_listening = handle.clone();
    let url = format!("http://{addr}{path}");

    tokio::spawn(async move {
        if handle_for_listening.clone().listening().await.is_some() {
            tracing::info!("GraphQL endpoint exposed at {url}");

            if let Some(mcp_url) = mcp_url {
                tracing::info!("MCP endpoint exposed at {mcp_url}");
            }

            server_runtime.on_ready(url);
        }
    });

    match tls {
        Some(tls) => {
            let rustls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(&tls.certificate, &tls.key)
                .await
                .map_err(crate::Error::CertificateError)?;

            axum_server::bind_rustls(addr, rustls_config)
                .handle(handle)
                .serve(app)
                .await
                .map_err(crate::Error::Server)?
        }
        None => axum_server::bind(addr)
            .handle(handle)
            .serve(app)
            .await
            .map_err(crate::Error::Server)?,
    }

    Ok(())
}

#[cfg(feature = "lambda")]
async fn lambda_bind(path: &str, router: Router<()>, mcp_url: Option<String>) -> crate::Result<()> {
    let app = tower::ServiceBuilder::new()
        .layer(engine_axum::lambda::LambdaLayer::default())
        .service(router);

    tracing::info!("GraphQL endpoint exposed at {path}");

    if let Some(mcp_url) = mcp_url {
        tracing::info!("MCP endpoint exposed at {mcp_url}");
    }

    lambda_http::run(app).await.expect("cannot start lambda http server");

    Ok(())
}

/// Waits for a termination signal and initiates a graceful shutdown of the server.
///
/// # Arguments
///
/// * `handle`: The handle for the server to manage graceful shutdown.
///
/// # Description
///
/// This function listens for termination signals (Ctrl+C or Unix termination signals)
/// and triggers a graceful shutdown of the server, allowing ongoing requests to complete
/// before shutting down.
async fn graceful_shutdown(handle: axum_server::Handle) {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutting down gracefully...");
    handle.graceful_shutdown(Some(std::time::Duration::from_secs(3)));
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
#[allow(dead_code)]
/// Response from the API containing graph information
pub struct ObjectStorageResponse {
    /// Account id of the owner of the referenced graph
    pub account_id: Ulid,
    /// The id of the graph
    pub graph_id: Ulid,
    /// The branch name
    pub branch: String,
    /// Grafbase id to uniquely identify the branch
    pub branch_id: Ulid,
    /// GraphQL SDL
    pub sdl: String,
    /// Current version's id generated by Grafbase
    pub version_id: Ulid,
}
