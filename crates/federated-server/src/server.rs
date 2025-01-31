mod access_logs;
mod cors;
mod csrf;
mod engine_reloader;
mod gateway;
mod graph_fetch_method;
mod graph_updater;
mod health;
mod state;
mod trusted_documents_client;

pub use graph_fetch_method::GraphFetchMethod;
pub use state::ServerState;

use runtime_local::wasi::hooks::{self, ComponentLoader, HooksWasi};
use ulid::Ulid;

use axum::{extract::State, response::IntoResponse, routing::get, Router};
use engine_axum::{
    middleware::{ResponseHookLayer, TelemetryLayer},
    websocket::{WebsocketAccepter, WebsocketService},
};
use engine_reloader::EngineReloader;
use gateway_config::{Config, TlsConfig};
use std::{net::SocketAddr, path::PathBuf};
use tokio::sync::mpsc;
use tokio::{signal, sync::watch};
use tower_http::cors::CorsLayer;

pub type ServerRouter<T> = Router<ServerState<T>>;

/// Start parameter for the gateway.
pub struct ServerConfig {
    /// The GraphQL endpoint listen address.
    pub listen_addr: Option<SocketAddr>,
    /// The gateway configuration.
    pub config_receiver: watch::Receiver<Config>,
    /// The config file path for hot reload.
    pub config_path: Option<PathBuf>,
    /// If true, watches changes to the config
    /// and reloads _some_ of the things.
    pub config_hot_reload: bool,
    /// The way of loading the graph for the gateway.
    pub fetch_method: GraphFetchMethod,
}

/// Trait for server runtime.
pub trait ServerRuntime: Send + Sync + 'static + Clone {
    /// Called after each request
    fn after_request(&self);
    /// Called when the server is ready and listening
    fn on_ready(&self, url: String);
    fn get_external_router<T>(&self) -> Option<ServerRouter<T>>;
}

impl ServerRuntime for () {
    fn after_request(&self) {}
    fn on_ready(&self, _url: String) {}
    fn get_external_router<T>(&self) -> Option<ServerRouter<T>> {
        None
    }
}

/// Starts the server and listens for incoming requests.
///
/// # Arguments
///
/// * `ServerConfig` - A configuration struct containing settings for the server, including:
///   - `listen_addr`: The address to listen on for incoming requests.
///   - `config`: The gateway configuration.
///   - `config_path`: Optional path for the config file for hot reload.
///   - `config_hot_reload`: A boolean indicating if the server should watch for config changes.
///   - `fetch_method`: The method used to load the graph for the gateway.
/// * `server_runtime`: An implementation of the `ServerRuntime` trait, providing hooks for request handling.
///
/// # Returns
///
/// This function returns a `Result` indicating success or failure. On success, it will return `Ok(())`.
///
/// # Errors
///
/// This function may return errors related to configuration loading, server binding, or request handling.
pub async fn serve(
    ServerConfig {
        listen_addr,
        config_receiver,
        config_path,
        fetch_method,
        config_hot_reload,
    }: ServerConfig,
    server_runtime: impl ServerRuntime,
) -> crate::Result<()> {
    let config = config_receiver.borrow().clone();
    let path = config.graph.path.as_deref().unwrap_or("/graphql");

    let meter = grafbase_telemetry::metrics::meter_from_global_provider();
    let pending_logs_counter = meter.i64_up_down_counter("grafbase.gateway.access_log.pending").build();

    let (access_log_sender, access_log_receiver) =
        hooks::create_log_channel(config.gateway.access_logs.lossy_log(), pending_logs_counter.clone());

    let hooks_loader = config
        .hooks
        .clone()
        .map(ComponentLoader::hooks)
        .transpose()
        .map_err(|e| crate::Error::InternalError(e.to_string()))?
        .flatten();

    let max_pool_size = config.hooks.as_ref().and_then(|config| config.max_pool_size);
    let hooks = HooksWasi::new(hooks_loader, max_pool_size, &meter, access_log_sender.clone()).await;

    let graph_stream = fetch_method.into_stream().await?;

    let update_handler = EngineReloader::spawn(
        config_receiver,
        graph_stream,
        config_hot_reload.then_some(config_path).flatten(),
        hooks.clone(),
        access_log_sender.clone(),
    )
    .await?;

    let gateway = update_handler.engine_watcher();

    if config.gateway.access_logs.enabled {
        access_logs::start(&config.gateway.access_logs, access_log_receiver, pending_logs_counter)?;
    }

    let (websocket_sender, websocket_receiver) = mpsc::channel(16);
    let websocket_accepter = WebsocketAccepter::new(websocket_receiver, gateway.clone());

    tokio::spawn(websocket_accepter.handler());

    let cors = match config.cors {
        Some(cors_config) => cors::generate(cors_config),
        None => CorsLayer::permissive(),
    };

    let state = ServerState::new(
        gateway,
        config.request_body_limit.bytes().max(0) as usize,
        server_runtime.clone(),
    );

    let mut router = server_runtime
        .get_external_router()
        .unwrap_or_default()
        .route(path, get(engine_execute).post(engine_execute))
        .route_service("/ws", WebsocketService::new(websocket_sender))
        .layer(ResponseHookLayer::new(hooks))
        .layer(TelemetryLayer::new(
            grafbase_telemetry::metrics::meter_from_global_provider(),
            listen_addr,
        ))
        .layer(cors);

    if config.health.enabled {
        if let Some(listen) = config.health.listen {
            tokio::spawn(health::bind_health_endpoint(
                listen,
                config.tls.clone(),
                config.health,
                state.clone(),
            ));
        } else {
            router = router.route(&config.health.path, get(health::health));
        }
    }

    let mut router = router.with_state(state);

    if config.csrf.enabled {
        router = csrf::inject_layer(router);
    }

    cfg_if::cfg_if! {
        if #[cfg(feature = "lambda")] {
            let result = lambda_bind(path, router).await;
        } else {
            use std::net::{IpAddr, Ipv4Addr};

            const DEFAULT_LISTEN_ADDRESS: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5000);

            let addr = listen_addr
                .or(config.network.listen_address)
                .unwrap_or(DEFAULT_LISTEN_ADDRESS);

            let result = bind(addr, path, router, config.tls.as_ref(), server_runtime).await;
        }
    }

    // Once all pending requests have been dealt with, we shutdown everything else left (telemetry, logs)
    if config.gateway.access_logs.enabled {
        access_log_sender.graceful_shutdown().await;
    }

    result
}

#[cfg_attr(feature = "lambda", allow(unused))]
async fn bind(
    addr: SocketAddr,
    path: &str,
    router: Router<()>,
    tls: Option<&TlsConfig>,
    server_runtime: impl ServerRuntime,
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
async fn lambda_bind(path: &str, router: Router<()>) -> crate::Result<()> {
    let app = tower::ServiceBuilder::new()
        .layer(engine_axum::lambda::LambdaLayer::default())
        .service(router);

    tracing::info!("GraphQL endpoint exposed at {path}");
    lambda_http::run(app).await.expect("cannot start lambda http server");

    Ok(())
}

/// Executes a GraphQL request against the registered engine.
///
/// # Arguments
///
/// * `State(state)`: The server state containing the gateway.
/// * `request`: The incoming Axum request containing the GraphQL query.
///
/// # Returns
///
/// This function returns an implementation of `IntoResponse`, which represents
/// the HTTP response to be sent back to the client.
///
/// # Errors
///
/// If there are no subgraphs registered, an internal server error response will
/// be returned.
async fn engine_execute<T>(State(state): State<ServerState<T>>, request: axum::extract::Request) -> impl IntoResponse
where
    T: ServerRuntime,
{
    let engine = state.gateway.borrow().clone();

    let response = engine_axum::execute(engine, request, state.request_body_limit_bytes).await;

    // lambda must flush the trace events here, otherwise the
    // function might fall asleep and the events are pending until
    // the next wake-up.
    //
    // read more: https://github.com/open-telemetry/opentelemetry-lambda/blob/main/docs/design_proposal.md
    #[cfg(feature = "lambda")]
    state.server_runtime.after_request();

    response
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
pub struct GdnResponse {
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
