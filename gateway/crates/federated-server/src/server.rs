mod cors;
mod csrf;
mod engine;
mod gateway;
mod graph_fetch_method;
mod graph_updater;
mod state;
mod trusted_documents_client;

pub use graph_fetch_method::GraphFetchMethod;

use crate::config::{Config, TlsConfig};
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

#[cfg(not(feature = "lambda"))]
async fn bind(addr: SocketAddr, path: &str, router: Router, tls: Option<TlsConfig>) -> Result<(), crate::Error> {
    let app = router.into_make_service();

    match tls {
        Some(ref tls) => {
            tracing::info!("starting the Grafbase gateway at https://{addr}{path}");

            let rustls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(&tls.certificate, &tls.key)
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

#[cfg(feature = "lambda")]
async fn bind(_: SocketAddr, path: &str, router: Router, _: Option<TlsConfig>) -> Result<(), crate::Error> {
    let app = tower::ServiceBuilder::new()
        .layer(axum_aws_lambda::LambdaLayer::default())
        .service(router);

    tracing::info!("starting the Grafbase Lambda gateway in {path}");

    lambda_http::run(app).await.expect("unable to start lambda http server");

    Ok(())
}
