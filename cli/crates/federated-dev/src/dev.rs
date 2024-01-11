use async_graphql::{EmptySubscription, Schema};
use async_graphql_axum::GraphQL;
use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, HeaderValue},
    response::{Html, IntoResponse},
    routing::get,
    Json,
};
use common::environment::Environment;
use handlebars::Handlebars;
use serde_json::json;
use std::time::Duration;
use tokio::sync::{mpsc, watch};
use tower_http::cors::CorsLayer;

use crate::{dev::gateway_nanny::GatewayNanny, ConfigReceiver};

use self::{
    bus::{AdminBus, ComposeBus, GatewayReceiver, RefreshBus},
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
    gateway: GatewayReceiver,
}

pub(super) async fn run(port: u16, expose: bool, config: ConfigReceiver) -> Result<(), crate::Error> {
    log::trace!("starting the federated dev server");

    let (graph_sender, graph_receiver) = watch::channel(None);
    let (refresh_sender, refresh_receiver) = mpsc::channel(16);
    let (compose_sender, compose_receiver) = mpsc::channel(16);
    let (gateway_sender, gateway) = watch::channel(None);

    let compose_bus = ComposeBus::new(graph_sender, refresh_sender, compose_sender.clone(), compose_receiver);
    let refresh_bus = RefreshBus::new(refresh_receiver, compose_sender.clone());
    let admin_bus = AdminBus::new(compose_sender.clone());

    let composer = Composer::new(compose_bus);
    tokio::spawn(composer.handler());

    let refresher = Refresher::new(refresh_bus);
    tokio::spawn(refresher.handler());

    let nanny = GatewayNanny::new(graph_receiver, config, gateway_sender);
    tokio::spawn(nanny.handler());

    let ticker = Ticker::new(REFRESH_INTERVAL, compose_sender);
    tokio::spawn(ticker.handler());

    let schema = Schema::build(admin::QueryRoot, admin::MutationRoot, EmptySubscription)
        .data(admin_bus)
        .finish();

    let environment = Environment::get();
    let static_asset_path = environment.user_dot_grafbase_path.join("static");

    let app = axum::Router::new()
        .route("/admin", get(admin).post_service(GraphQL::new(schema)))
        .route("/graphql", get(engine_get).post(engine_post))
        .nest_service("/static", tower_http::services::ServeDir::new(static_asset_path))
        .layer(CorsLayer::permissive())
        .with_state(ProxyState {
            admin_pathfinder_html: Html(render_pathfinder(port, "/admin")),
            gateway,
        });

    let host = if expose {
        format!("0.0.0.0:{port}")
    } else {
        format!("127.0.0.1:{port}")
    };
    let address: std::net::SocketAddr = host.parse().expect("we just defined it above, it _must work_");

    let listener = tokio::net::TcpListener::bind(&address).await.unwrap();
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
    Query(request): Query<engine::Request>,
    headers: HeaderMap,
    State(ProxyState { gateway, .. }): State<ProxyState>,
) -> impl IntoResponse {
    handle_engine_request(request, gateway, headers).await
}

async fn engine_post(
    State(ProxyState { gateway, .. }): State<ProxyState>,
    headers: HeaderMap,
    Json(request): Json<engine::Request>,
) -> impl IntoResponse {
    handle_engine_request(request, gateway, headers).await
}

async fn handle_engine_request(
    request: engine::Request,
    gateway: GatewayReceiver,
    headers: HeaderMap,
) -> impl IntoResponse {
    let headers = headers
        .into_iter()
        .map(|(name, value)| {
            (
                name.map(|name| name.to_string()).unwrap_or_default(),
                String::from_utf8_lossy(value.as_bytes()).to_string(),
            )
        })
        .collect();

    let Some(gateway) = gateway.borrow().clone() else {
        return Json(json!({
            "errors": [{"message": "there are no subgraphs registered currently"}]
        }))
        .into_response();
    };

    let result = gateway.execute(request, headers, serde_json::to_vec).await;

    match result {
        Ok(response) => (
            [(
                header::CONTENT_TYPE,
                HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
            )],
            response.bytes,
        )
            .into_response(),
        Err(error) => Json(json!({
            "errors": [{"message": error.to_string()}]
        }))
        .into_response(),
    }
}
