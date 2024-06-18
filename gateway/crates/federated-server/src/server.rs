mod cors;
mod csrf;
mod engine;
mod gateway;
mod graph_fetch_method;
#[cfg(not(feature = "lambda"))]
mod graph_updater;
mod otel;
mod state;
mod trusted_documents_client;

use grafbase_tracing::gql_response_status::GraphqlResponseStatus;
pub use graph_fetch_method::GraphFetchMethod;
pub use otel::{OtelReload, OtelTracing};
use tokio::sync::watch;
use tracing::Level;
use ulid::Ulid;

use crate::config::Config;
use crate::config::TlsConfig;
use axum::{routing::get, Router};
use axum_server as _;
use engine_v2_axum::websocket::{WebsocketAccepter, WebsocketService};
use grafbase_tracing::span::GRAFBASE_TARGET;
use state::ServerState;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::sync::mpsc;
use tower_http::cors::CorsLayer;

use self::gateway::GatewayConfig;

const DEFAULT_LISTEN_ADDRESS: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5000);

/// Starts the self-hosted Grafbase gateway. If started with a schema path, will
/// not connect our API for changes in the schema and if started without, we poll
/// the schema registry every ten second for changes.
pub async fn serve(
    listen_addr: Option<SocketAddr>,
    config: Config,
    fetch_method: GraphFetchMethod,
    otel_tracing: Option<OtelTracing>,
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
    fetch_method.start(
        GatewayConfig {
            enable_introspection: config.graph.introspection,
            operation_limits: config.operation_limits,
            authentication: config.authentication,
            subgraphs: config.subgraphs,
            default_headers: config.headers,
            trusted_documents: config.trusted_documents,
        },
        otel_reload,
        sender,
    )?;

    let (websocket_sender, websocket_receiver) = mpsc::channel(16);
    let websocket_accepter = WebsocketAccepter::new(websocket_receiver, gateway.clone());

    tokio::spawn(websocket_accepter.handler());

    let cors = match config.cors {
        Some(cors_config) => cors::generate(cors_config),
        None => CorsLayer::permissive(),
    };

    let state = ServerState::new(gateway.clone(), otel_tracer_provider);

    // HACK: Wait for the engine to be ready. This ensures we did reload OTEL providers if necessary
    // as we need all resources attributes to be present before creating the tracing layer.
    tracing::event!(target: GRAFBASE_TARGET, Level::INFO, "Waiting for engine to be ready...");
    gateway.changed().await.ok();
    let mut router = Router::new()
        .route(path, get(engine::get).post(engine::post))
        .route_service("/ws", WebsocketService::new(websocket_sender))
        .layer(grafbase_tracing::tower::layer(
            grafbase_tracing::metrics::meter_from_global_provider(),
        ))
        .layer(axum::middleware::map_response(
            |mut response: axum::response::Response<_>| async {
                response.headers_mut().remove(GraphqlResponseStatus::header_name());
                response
            },
        ))
        .layer(cors)
        .with_state(state);

    if config.csrf.enabled {
        router = csrf::inject_layer(router);
    }

    bind(addr, path, router, config.tls).await?;

    Ok(())
}

#[cfg(not(feature = "lambda"))]
async fn bind(addr: SocketAddr, path: &str, router: Router<()>, tls: Option<TlsConfig>) -> crate::Result<()> {
    let app = router.into_make_service();

    match tls {
        Some(ref tls) => {
            tracing::info!(target: GRAFBASE_TARGET, "GraphQL endpoint exposed at https://{addr}{path}");

            let rustls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(&tls.certificate, &tls.key)
                .await
                .map_err(crate::Error::CertificateError)?;

            axum_server::bind_rustls(addr, rustls_config)
                .serve(app)
                .await
                .map_err(crate::Error::Server)?
        }
        None => {
            tracing::info!(target: GRAFBASE_TARGET, "GraphQL endpoint exposed at http://{addr}{path}");
            axum_server::bind(addr).serve(app).await.map_err(crate::Error::Server)?
        }
    }

    Ok(())
}

#[cfg(feature = "lambda")]
async fn bind(_: SocketAddr, path: &str, router: Router<()>, _: Option<TlsConfig>) -> crate::Result<()> {
    let app = tower::ServiceBuilder::new()
        .layer(axum_aws_lambda::LambdaLayer::default())
        .service(router);

    tracing::info!(target: GRAFBASE_TARGET, "GraphQL endpoint exposed at {path}");
    lambda_http::run(app).await.expect("cannot start lambda http server");

    Ok(())
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
