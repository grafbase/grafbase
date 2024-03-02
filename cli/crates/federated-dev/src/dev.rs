use crate::{dev::gateway_nanny::GatewayNanny, ConfigWatcher};
use async_graphql::{EmptySubscription, Schema};
use async_graphql_axum::GraphQL;
use axum::{
    body::Bytes,
    extract::{RawQuery, State},
    http::HeaderMap,
    response::{Html, IntoResponse},
    routing::get,
    Json,
};
use common::environment::Environment;
use engine_v2::{HttpGraphqlRequest, WebsocketAccepter, WebsocketService};
use graphql_composition::FederatedGraph;
use handlebars::Handlebars;
use serde_json::json;
use std::{net::SocketAddr, time::Duration};
use tokio::sync::{mpsc, watch};
use tower_http::cors::CorsLayer;

use self::{
    bus::{AdminBus, ComposeBus, EngineWatcher, RefreshBus},
    composer::Composer,
    refresher::Refresher,
    ticker::Ticker,
};

mod admin;
mod bus;
mod composer;
mod gateway_nanny;
mod refresher;
mod ticker;

const REFRESH_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Clone)]
struct ProxyState {
    admin_pathfinder_html: Html<String>,
    engine: EngineWatcher,
}

pub(super) async fn run(
    listen_address: SocketAddr,
    config: ConfigWatcher,
    graph: Option<FederatedGraph>,
) -> Result<(), crate::Error> {
    log::trace!("starting the federated dev server");

    let (gateway_sender, engine) = watch::channel(gateway_nanny::new_gateway(graph, &config.borrow()));
    let (websocket_sender, websocket_receiver) = mpsc::channel(16);

    let websocket_accepter = WebsocketAccepter::new(websocket_receiver, engine.clone());
    tokio::spawn(websocket_accepter.handler());

    let admin_schema = if engine.borrow().is_some() {
        log::debug!("Disabling subgraph composition, federated graph was provided.");
        Schema::build(admin::QueryRoot, admin::MutationRoot, EmptySubscription)
            .data(AdminBus::new_static())
            .finish()
    } else {
        let (graph_sender, graph_receiver) = watch::channel(None);
        let (compose_sender, compose_receiver) = mpsc::channel(16);
        let (refresh_sender, refresh_receiver) = mpsc::channel(16);
        let refresh_bus = RefreshBus::new(refresh_receiver, compose_sender.clone());
        let compose_bus = ComposeBus::new(graph_sender, refresh_sender, compose_sender.clone(), compose_receiver);
        let composer = Composer::new(compose_bus);
        tokio::spawn(composer.handler());

        let ticker = Ticker::new(REFRESH_INTERVAL, compose_sender.clone());
        tokio::spawn(ticker.handler());

        let refresher = Refresher::new(refresh_bus);
        tokio::spawn(refresher.handler());

        let nanny = GatewayNanny::new(graph_receiver, config, gateway_sender);
        tokio::spawn(nanny.handler());

        let admin_bus = AdminBus::new_dynamic(compose_sender);

        Schema::build(admin::QueryRoot, admin::MutationRoot, EmptySubscription)
            .data(admin_bus)
            .finish()
    };

    let environment = Environment::get();
    let static_asset_path = environment.user_dot_grafbase_path.join("static");

    let app = axum::Router::new()
        .route("/admin", get(admin).post_service(GraphQL::new(admin_schema)))
        .route("/graphql", get(engine_get).post(engine_post))
        .route_service("/ws", WebsocketService::new(websocket_sender))
        .nest_service("/static", tower_http::services::ServeDir::new(static_asset_path))
        .layer(CorsLayer::permissive())
        .layer(grafbase_tracing::tower::layer())
        .with_state(ProxyState {
            admin_pathfinder_html: Html(render_pathfinder(listen_address.port(), "/admin")),
            engine,
        });

    let listener = tokio::net::TcpListener::bind(&listen_address).await.unwrap();
    axum::serve(listener, app)
        .await
        .map_err(|error| crate::Error::internal(error.to_string()))?;

    Ok(())
}

fn render_pathfinder(port: u16, graphql_url: &str) -> String {
    let mut handlebars = Handlebars::new();
    let template = include_str!("../../server/templates/pathfinder.hbs");

    handlebars
        .register_template_string("pathfinder.html", template)
        .expect("must be valid");

    let asset_url = format!("http://127.0.0.1:{port}/static");

    handlebars
        .render(
            "pathfinder.html",
            &json!({
                "ASSET_URL": asset_url,
                "GRAPHQL_URL": graphql_url
            }),
        )
        .expect("must render")
}

#[allow(clippy::unused_async)]
async fn admin(
    State(ProxyState {
        admin_pathfinder_html, ..
    }): State<ProxyState>,
) -> impl IntoResponse {
    admin_pathfinder_html
}

async fn engine_get(
    headers: HeaderMap,
    RawQuery(query): RawQuery,
    State(ProxyState { engine, .. }): State<ProxyState>,
) -> impl IntoResponse {
    handle_engine_request(
        engine,
        headers,
        HttpGraphqlRequest::Query(query.unwrap_or_default().into()),
    )
    .await
}

async fn engine_post(
    State(ProxyState { engine, .. }): State<ProxyState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    handle_engine_request(engine, headers, HttpGraphqlRequest::JsonBodyBytes(body)).await
}

async fn handle_engine_request(
    engine: EngineWatcher,
    headers: HeaderMap,
    request: HttpGraphqlRequest<'_>,
) -> impl IntoResponse {
    log::debug!("engine request received");
    let Some(engine) = engine.borrow().clone() else {
        return Json(json!({
            "errors": [{"message": "there are no subgraphs registered currently"}]
        }))
        .into_response();
    };
    let ray_id = ulid::Ulid::new().to_string();
    engine.execute(headers, &ray_id, request).await.into_response()
}
