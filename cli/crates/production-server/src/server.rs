mod cors;
mod csrf;
mod engine;
mod gateway;
mod graph_updater;
mod pathfinder;
mod state;

use crate::{config::Config, GraphFetchMethod};
use axum::{response::Html, routing::get, Router};
use axum_server::tls_rustls::RustlsConfig;
use gateway_v2::local_server::{WebsocketAccepter, WebsocketService};
use state::ServerState;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};
use tokio::sync::{mpsc, watch};
use tower_http::{cors::CorsLayer, services::ServeDir};

use self::graph_updater::GraphUpdater;

pub(super) async fn serve(
    listen_addr: Option<SocketAddr>,
    config: Config,
    graph: GraphFetchMethod,
) -> crate::Result<()> {
    let path = config.graph.path.as_deref().unwrap_or("/graphql");

    let addr = listen_addr
        .or(config.network.listen_address)
        .unwrap_or(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 4000));

    let url = if config.tls.is_some() {
        format!("https://{addr}")
    } else {
        format!("http://{addr}")
    };

    let (sender, gateway) = watch::channel(None);

    match graph {
        GraphFetchMethod::FromApi {
            access_token,
            graph_name,
            branch,
        } => {
            tokio::spawn(async move {
                let mut updater = GraphUpdater::new(&graph_name, branch.as_deref(), access_token, sender)?
                    .enable_introspection(config.graph.introspection);

                if let Some(operation_limits) = config.operation_limits {
                    updater = updater.with_operation_limits(operation_limits);
                }

                if let Some(auth_config) = config.authentication {
                    updater = updater.with_authentication(auth_config);
                }

                updater.poll().await;

                Ok::<_, crate::Error>(())
            });
        }
        GraphFetchMethod::FromLocal { federated_schema } => {
            let gateway = gateway::generate(
                &federated_schema,
                config.operation_limits,
                config.authentication,
                config.graph.introspection,
            )?;

            sender.send(Some(Arc::new(gateway)))?;
        }
    }

    let (websocket_sender, websocket_receiver) = mpsc::channel(16);
    let websocket_accepter = WebsocketAccepter::new(websocket_receiver, gateway.clone());

    tokio::spawn(websocket_accepter.handler());

    let state = ServerState {
        gateway,
        pathfinder_html: Html(pathfinder::render(&url, path)),
    };

    let static_asset_path = "/home/pimeys/.grafbase/static";

    let cors = match config.cors {
        Some(cors_config) => cors::generate(cors_config),
        None => CorsLayer::permissive(),
    };

    let mut router = Router::new()
        .route(path, get(engine::get).post(engine::post))
        .route("/", get(pathfinder::get))
        .route_service("/ws", WebsocketService::new(websocket_sender))
        .nest_service("/static", ServeDir::new(static_asset_path))
        .layer(cors)
        .with_state(state);

    if config.csrf.enabled {
        router = csrf::inject_layer(router);
    }

    let app = router.into_make_service();

    match config.tls {
        Some(ref tls) => {
            tracing::info!("starting the Grafbase gateway in https://{addr}{path}");

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
