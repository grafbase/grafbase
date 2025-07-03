mod cors;
mod csrf;
mod engine_reloader;
mod events;
mod gateway;
mod graph_fetch_method;
mod graph_updater;
mod health;
mod public_auth_metadata;
mod state;
mod trusted_documents_client;

use self::events::UpdateEvent;
use self::public_auth_metadata::*;
pub(crate) use gateway::CreateExtensionCatalogError;
use gateway::{EngineWatcher, create_extension_catalog::create_extension_catalog};
pub use graph_fetch_method::GraphFetchMethod;
use runtime::{authentication::Authenticate as _, extension::HooksExtension};
pub use state::ServerState;
use tokio_util::sync::CancellationToken;
use ulid::Ulid;

use axum::{Router, extract::State, response::IntoResponse, routing::get};
use engine_axum::{
    middleware::{HooksLayer, TelemetryLayer},
    websocket::{WebsocketAccepter, WebsocketService},
};
use engine_reloader::{EngineReloaderConfig, GatewayEngineReloader};
use gateway_config::{Config, TlsConfig};
use std::{net::SocketAddr, path::PathBuf};
use tokio::{
    signal,
    sync::{mpsc, watch},
};
use tower_http::{
    compression::{CompressionLayer, DefaultPredicate, Predicate as _, predicate::NotForContentType},
    cors::CorsLayer,
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

    let (router, ct) = router(
        config,
        update_handler.engine_watcher(),
        server_runtime.clone(),
        hooks,
        |router| {
            // Currently we're doing those after CORS handling in the request as we don't care
            // about pre-flight requests.
            let telemetry_layer = TelemetryLayer::new(
                grafbase_telemetry::metrics::meter_from_global_provider(),
                Some(listen_address),
            );

            router.layer(telemetry_layer)
        },
    )
    .await?;

    cfg_if::cfg_if! {
        if #[cfg(feature = "lambda")] {
            let result = lambda_bind(&path, router, mcp_url).await;
        } else {
            let result = bind(listen_address, &path, router, tls.as_ref(), server_runtime, mcp_url).await;
        }
    }

    if let Some(ct) = ct {
        ct.cancel();
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

pub async fn router<R: engine::Runtime, SR: ServerRuntime, H: HooksExtension>(
    config: gateway_config::Config,
    engine: EngineWatcher<R>,
    server_runtime: SR,
    hooks: H,
    inject_layers_before_cors: impl FnOnce(axum::Router<()>) -> axum::Router<()>,
) -> crate::Result<(axum::Router, Option<CancellationToken>)> {
    let path = &config.graph.path;
    let websocket_path = &config.graph.websocket_path;

    let (websocket_sender, websocket_receiver) = mpsc::channel(16);
    let websocket_accepter = WebsocketAccepter::new(websocket_receiver, engine.clone());

    tokio::spawn(websocket_accepter.handler());

    let cors = match config.cors {
        Some(ref cors_config) => cors::generate(cors_config),
        None => CorsLayer::permissive(),
    };

    let state = ServerState::new(
        engine.clone(),
        config.request_body_limit.bytes().max(0) as usize,
        server_runtime.clone(),
    );

    let mut router = server_runtime
        .base_router()
        .unwrap_or_default()
        .route(path, get(graphql_execute).post(graphql_execute))
        .route_service(websocket_path, WebsocketService::new(websocket_sender))
        .with_state(state);

    let ct = match &config.mcp {
        Some(mcp_config) if mcp_config.enabled => {
            let (mcp_router, ct) = grafbase_mcp::router(&engine, mcp_config);
            router = router.merge(mcp_router);
            ct
        }
        _ => None,
    };

    for public_metadata_endpoint in engine
        .borrow()
        .runtime
        .authentication()
        .public_metadata_endpoints()
        .await
        .unwrap_or_default()
    {
        router = router.route(
            &public_metadata_endpoint.path,
            get(public_metadata_handler(
                public_metadata_endpoint.response_body.into(),
                public_metadata_endpoint.headers,
            )),
        );
    }

    let mut router = inject_layers_before_cors(router)
        .layer(HooksLayer::new(hooks))
        // Streaming and compression doesn't really work well today. Had a panic deep inside stream
        // unfold. Furthermore there seem to be issues with it as pointed out by Apollo's router
        // team:
        // https://github.com/tower-rs/tower-http/issues/292
        // They have copied the compression code and adjusted it, see PRs for:
        // https://github.com/apollographql/router/issues/1572
        // We'll need to see what we do. For now I'm disabling it as it's not important enough
        // right now.
        .layer(CompressionLayer::new().compress_when(DefaultPredicate::new().and(
            NotForContentType::const_new("multipart/mixed").and(NotForContentType::const_new("text/event-stream")),
        )))
        .layer(cors);

    if config.csrf.enabled {
        router = csrf::inject_layer(router, &config.csrf);
    }

    if config.health.enabled {
        if let Some(listen) = config.health.listen {
            tokio::spawn(health::bind_health_endpoint(listen, config.tls.clone(), config.health));
        } else {
            router = router.route(&config.health.path, get(health::health));
        }
    }

    Ok((router, ct))
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
async fn graphql_execute<R: engine::Runtime, SR: ServerRuntime>(
    State(state): State<ServerState<R, SR>>,
    request: axum::extract::Request,
) -> impl IntoResponse {
    let engine = state.engine.borrow().clone();

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
