mod bind;
mod cors;
mod csrf;
mod engine;
mod gateway;
mod graph_fetch_method;
#[cfg(not(feature = "lambda"))]
mod graph_updater;
mod state;
mod trusted_documents_client;

pub use graph_fetch_method::GraphFetchMethod;

use crate::config::Config;
use axum::{routing::get, Router};
use axum_server as _;
use gateway_v2::local_server::{WebsocketAccepter, WebsocketService};
use state::ServerState;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::sync::mpsc;
use tower_http::cors::CorsLayer;

use self::gateway::GatewayConfig;

const DEFAULT_LISTEN_ADDRESS: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5000);

pub(super) async fn serve(
    listen_addr: Option<SocketAddr>,
    config: Config,
    fetch_method: GraphFetchMethod,
) -> crate::Result<()> {
    let path = config.graph.path.as_deref().unwrap_or("/graphql");

    let addr = listen_addr
        .or(config.network.listen_address)
        .unwrap_or(DEFAULT_LISTEN_ADDRESS);

    let gateway = fetch_method.into_gateway(GatewayConfig {
        enable_introspection: config.graph.introspection,
        operation_limits: config.operation_limits,
        authentication: config.authentication,
        subgraphs: config.subgraphs,
        default_headers: config.headers,
        trusted_documents: config.trusted_documents,
    })?;

    let (websocket_sender, websocket_receiver) = mpsc::channel(16);
    let websocket_accepter = WebsocketAccepter::new(websocket_receiver, gateway.clone());

    tokio::spawn(websocket_accepter.handler());

    let cors = match config.cors {
        Some(cors_config) => cors::generate(cors_config),
        None => CorsLayer::permissive(),
    };

    let router = Router::new()
        .route(path, get(engine::get).post(engine::post))
        .route_service("/ws", WebsocketService::new(websocket_sender))
        .layer(cors)
        .layer(grafbase_tracing::tower::layer());

    bind::bind(bind::BindConfig {
        addr,
        path,
        router,
        gateway,
        tls: config.tls,
        telemetry: config.telemetry,
        csrf: config.csrf.enabled,
    })
    .await?;

    Ok(())
}
