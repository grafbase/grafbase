mod cors;
mod csrf;
mod engine;
mod gateway;
mod graph_fetch_method;
#[cfg(not(feature = "lambda"))]
mod graph_updater;
mod health;
mod otel;
mod state;
mod trusted_documents_client;

use grafbase_telemetry::gql_response_status::GraphqlResponseStatus;
pub use graph_fetch_method::GraphFetchMethod;
pub use otel::{OtelReload, OtelTracing};
use tokio::sync::watch;
use tracing::Level;
use ulid::Ulid;

use axum::{routing::get, Router};
use axum_server as _;
use engine_v2_axum::websocket::{WebsocketAccepter, WebsocketService};
use gateway_config::{Config, TlsConfig};
use grafbase_telemetry::span::GRAFBASE_TARGET;
use state::ServerState;
use std::{
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
    /// The opentelemetry tracer.
    pub otel_tracing: Option<OtelTracing>,
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
        otel_tracing,
        config_hot_reload,
    }: ServerConfig,
) -> crate::Result<()> {
    let path = config.graph.path.as_deref().unwrap_or("/graphql");

    let addr = listen_addr
        .or(config.network.listen_address)
        .unwrap_or(DEFAULT_LISTEN_ADDRESS);

    let (otel_tracer_provider, otel_reload) = otel_tracing
        .map(|otel| {
            (
                Some(otel.tracer_provider),
                Some((otel.reload_trigger, otel.reload_ack_receiver)),
            )
        })
        .unwrap_or((None, None));

    let (sender, mut gateway) = watch::channel(None);
    gateway.mark_unchanged();

    fetch_method
        .start(
            &config,
            config_hot_reload.then_some(config_path).flatten(),
            otel_reload,
            sender,
        )
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
        otel_tracer_provider,
        config.request_body_limit.bytes().max(0) as usize,
    );

    // HACK: Wait for the engine to be ready. This ensures we did reload OTEL providers if necessary
    // as we need all resources attributes to be present before creating the tracing layer.
    tracing::event!(target: GRAFBASE_TARGET, Level::DEBUG, "waiting for engine to be ready...");
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

    bind(addr, path, router, config.tls.as_ref()).await?;

    Ok(())
}

#[cfg(not(feature = "lambda"))]
async fn bind(addr: SocketAddr, path: &str, router: Router<()>, tls: Option<&TlsConfig>) -> crate::Result<()> {
    let app = router.into_make_service();

    let handle = axum_server::Handle::new();

    // Spawn a task to gracefully shutdown server.
    tokio::spawn(graceful_shutdown(handle.clone()));

    match tls {
        Some(tls) => {
            tracing::info!(target: GRAFBASE_TARGET, "GraphQL endpoint exposed at https://{addr}{path}");

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
            tracing::info!(target: GRAFBASE_TARGET, "GraphQL endpoint exposed at http://{addr}{path}");
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
async fn bind(_: SocketAddr, path: &str, router: Router<()>, _: Option<&TlsConfig>) -> crate::Result<()> {
    let app = tower::ServiceBuilder::new()
        .layer(axum_aws_lambda::LambdaLayer::default())
        .service(router);

    tracing::info!(target: GRAFBASE_TARGET, "GraphQL endpoint exposed at {path}");
    lambda_http::run(app).await.expect("cannot start lambda http server");

    Ok(())
}

#[allow(unused)] // I profoundly despise those not(lambda) feature flags...
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

    tracing::info!(target: GRAFBASE_TARGET, "Shutting down gracefully...");
    grafbase_telemetry::graceful_shutdown();
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
