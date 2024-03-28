mod cors;
mod csrf;
mod engine;
mod gateway;
mod graph_fetch_method;
#[cfg(not(feature = "lambda"))]
mod graph_updater;
mod state;
mod trusted_documents_client;

use grafbase_tracing::otel::opentelemetry_sdk::trace::TracerProvider;
pub use graph_fetch_method::GraphFetchMethod;

use crate::config::Config;
use crate::config::TlsConfig;
use axum::{routing::get, Router};
use axum_server as _;
use gateway_v2::local_server::{WebsocketAccepter, WebsocketService};
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
    provider: Option<TracerProvider>,
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

    let state = ServerState::new(gateway, provider);

    let mut router = Router::new()
        .route(path, get(engine::get).post(engine::post))
        .route_service("/ws", WebsocketService::new(websocket_sender))
        .layer(cors)
        .layer(grafbase_tracing::tower::layer())
        .with_state(state);

    if config.csrf.enabled {
        router = csrf::inject_layer(router);
    }

    cfg_if::cfg_if! {
        if #[cfg(feature = "lambda")] {
            lambda_bind(addr, path, router, config.tls).await?;
        } else {
            bind(addr, path, router, config.tls).await?;
        }
    }

    Ok(())
}

async fn bind(addr: SocketAddr, path: &str, router: Router<()>, tls: Option<TlsConfig>) -> crate::Result<()> {
    let app = router.into_make_service();

    match tls {
        Some(ref tls) => {
            tracing::info!(target: GRAFBASE_TARGET, "starting the Grafbase gateway at https://{addr}{path}");

            let rustls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(&tls.certificate, &tls.key)
                .await
                .map_err(crate::Error::CertificateError)?;

            axum_server::bind_rustls(addr, rustls_config)
                .serve(app)
                .await
                .map_err(crate::Error::Server)?
        }
        None => {
            tracing::info!(target: GRAFBASE_TARGET, "starting the Grafbase gateway in http://{addr}{path}");
            axum_server::bind(addr).serve(app).await.map_err(crate::Error::Server)?
        }
    }

    Ok(())
}

#[cfg(feature = "lambda")]
async fn lambda_bind(_: SocketAddr, path: &str, router: Router<()>, _: Option<TlsConfig>) -> crate::Result<()> {
    let app = tower::ServiceBuilder::new()
        .layer(axum_aws_lambda::LambdaLayer::default())
        .service(router);

    tracing::info!(target: GRAFBASE_TARGET, "starting the Grafbase Lambda gateway in {path}");
    lambda_http::run(app).await.expect("cannot start lambda http server");

    Ok(())
}
