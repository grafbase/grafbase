mod cors;
mod csrf;
mod engine;
mod gateway;
mod graph_fetch_method;
mod graph_updater;
mod state;

pub use graph_fetch_method::GraphFetchMethod;

use crate::config::{Config, TlsConfig};
use axum::{routing::get, Router};
use axum_server::tls_rustls::RustlsConfig;
use gateway_v2::local_server::{WebsocketAccepter, WebsocketService};
use state::ServerState;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::sync::mpsc;
use tower_http::cors::CorsLayer;

pub(super) async fn serve(
    listen_addr: Option<SocketAddr>,
    config: Config,
    fetch_method: GraphFetchMethod,
) -> crate::Result<()> {
    let path = config.graph.path.as_deref().unwrap_or("/graphql");

    let addr = listen_addr
        .or(config.network.listen_address)
        .unwrap_or(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 4000));

    let gateway = fetch_method.into_gateway(
        config.graph.introspection,
        config.operation_limits,
        config.authentication,
    )?;

    let (websocket_sender, websocket_receiver) = mpsc::channel(16);
    let websocket_accepter = WebsocketAccepter::new(websocket_receiver, gateway.clone());

    tokio::spawn(websocket_accepter.handler());

    let state = ServerState { gateway };

    let cors = match config.cors {
        Some(cors_config) => cors::generate(cors_config),
        None => CorsLayer::permissive(),
    };

    let mut router = Router::new()
        .route(path, get(engine::get).post(engine::post))
        .route_service("/ws", WebsocketService::new(websocket_sender))
        .layer(cors)
        .layer(grafbase_tracing::tower::layer())
        .with_state(state);

    if config.csrf.enabled {
        router = csrf::inject_layer(router);
    }

    bind(addr, path, router, config.tls).await?;

    Ok(())
}

async fn bind(addr: SocketAddr, path: &str, router: Router, tls: Option<TlsConfig>) -> Result<(), crate::Error> {
    let app = router.into_make_service();

    match tls {
        Some(ref tls) => {
            tracing::info!("starting the Grafbase gateway at https://{addr}{path}");

            let rustls_config = RustlsConfig::from_pem_file(&tls.certificate, &tls.key)
                .await
                .map_err(crate::Error::CertificateError)?;

            axum_server::bind_rustls(addr, rustls_config)
                .serve(app)
                .await
                .map_err(crate::Error::Server)?
        }
        None => {
            tracing::info!("starting the Grafbase gateway in http://{addr}{path}");
            axum_server::bind(addr).serve(app).await.map_err(crate::Error::Server)?
        }
    }

    Ok(())
}
