mod cors;
mod csrf;
mod engine;
mod gateway;
mod graph_fetch_method;
mod graph_updater;
mod health;
mod state;
mod trusted_documents_client;

use grafbase_telemetry::gql_response_status::GraphqlResponseStatus;
pub use graph_fetch_method::GraphFetchMethod;
use tokio::sync::watch;
use ulid::Ulid;

use axum::{routing::get, Router};
use axum_server as _;
use engine_v2_axum::websocket::{WebsocketAccepter, WebsocketService};
use gateway_config::{Config, TlsConfig};
use state::ServerState;
use std::{
    future::Future,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    time::Duration,
};
use tokio::signal;
use tokio::sync::mpsc;
use tower_http::cors::CorsLayer;

const DEFAULT_LISTEN_ADDRESS: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5000);

/// Start parameter for the gateway.
pub struct ServerConfig {
    /// The GraphQL endpoint listen address.
    pub listen_addr: Option<SocketAddr>,
    /// The gateway configuration.
    pub config: Config,
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
    /// Called when the server shutdowns gracefully.
    fn graceful_shutdown(&self) -> impl Future<Output = ()> + Send;
    /// Called after each request
    fn after_request(&self);
}

impl ServerRuntime for () {
    async fn graceful_shutdown(&self) {}
    fn after_request(&self) {}
}

/// Starts the self-hosted Grafbase gateway. If started with a schema path, will
/// not connect our API for changes in the schema and if started without, we poll
/// the schema registry every ten second for changes.
pub async fn serve(
    ServerConfig {
        listen_addr,
        config,
        config_path,
        fetch_method,
        config_hot_reload,
    }: ServerConfig,
    server_runtime: impl ServerRuntime,
) -> crate::Result<()> {
    let path = config.graph.path.as_deref().unwrap_or("/graphql");

    let addr = listen_addr
        .or(config.network.listen_address)
        .unwrap_or(DEFAULT_LISTEN_ADDRESS);

    let (sender, mut gateway) = watch::channel(None);
    gateway.mark_unchanged();

    fetch_method
        .start(&config, config_hot_reload.then_some(config_path).flatten(), sender)
        .await?;

    let (websocket_sender, websocket_receiver) = mpsc::channel(16);
    let websocket_accepter = WebsocketAccepter::new(websocket_receiver, gateway.clone());

    tokio::spawn(websocket_accepter.handler());

    let cors = match config.cors {
        Some(cors_config) => cors::generate(cors_config),
        None => CorsLayer::permissive(),
    };

    let state = ServerState::new(
        gateway.clone(),
        config.request_body_limit.bytes().max(0) as usize,
        server_runtime.clone(),
    );

    tracing::debug!("Waiting for the engine to be ready...");
    gateway.changed().await.ok();

    let mut router = Router::new()
        .route(path, get(engine::execute).post(engine::execute))
        .route_service("/ws", WebsocketService::new(websocket_sender))
        .layer(grafbase_telemetry::tower::layer(
            grafbase_telemetry::metrics::meter_from_global_provider(),
            Some(addr),
        ))
        .layer(tower_http::timeout::RequestBodyTimeoutLayer::new(
            config.gateway.timeout.unwrap_or(DEFAULT_GATEWAY_TIMEOUT),
        ))
        .layer(axum::middleware::map_response(
            |mut response: axum::response::Response<_>| async {
                response.headers_mut().remove(GraphqlResponseStatus::header_name());
                response
            },
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
            lambda_bind(path, router).await?;
        } else {
            bind(addr, path, router, config.tls.as_ref()).await?;
        }
    }

    // Once all pending requests have been dealt with, we shutdown everything else left (telemetry)
    server_runtime.graceful_shutdown().await;

    Ok(())
}

#[cfg_attr(feature = "lambda", allow(unused))]
async fn bind(addr: SocketAddr, path: &str, router: Router<()>, tls: Option<&TlsConfig>) -> crate::Result<()> {
    let app = router.into_make_service();

    let handle = axum_server::Handle::new();

    // Spawn a task to gracefully shutdown server.
    tokio::spawn(graceful_shutdown(handle.clone()));

    match tls {
        Some(tls) => {
            tracing::info!("GraphQL endpoint exposed at https://{addr}{path}");

            let rustls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(&tls.certificate, &tls.key)
                .await
                .map_err(crate::Error::CertificateError)?;

            axum_server::bind_rustls(addr, rustls_config)
                .handle(handle)
                .serve(app)
                .await
                .map_err(crate::Error::Server)?
        }
        None => {
            tracing::info!("GraphQL endpoint exposed at http://{addr}{path}");
            axum_server::bind(addr)
                .handle(handle)
                .serve(app)
                .await
                .map_err(crate::Error::Server)?
        }
    }

    Ok(())
}

#[cfg(feature = "lambda")]
async fn lambda_bind(path: &str, router: Router<()>) -> crate::Result<()> {
    let app = tower::ServiceBuilder::new()
        .layer(axum_aws_lambda::LambdaLayer::default())
        .service(router);

    tracing::info!("GraphQL endpoint exposed at {path}");
    lambda_http::run(app).await.expect("cannot start lambda http server");

    Ok(())
}

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

#[derive(Debug, serde::Serialize, serde::Deserialize)]
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

const DEFAULT_GATEWAY_TIMEOUT: Duration = Duration::from_secs(30);
