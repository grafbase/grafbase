use async_graphql::{EmptySubscription, Schema};
use async_graphql_axum::GraphQL;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Json, Server,
};
use common::environment::Environment;
use handlebars::Handlebars;
use parser_sdl::federation::FederatedGraphConfig;
use serde_json::json;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tower_http::cors::CorsLayer;

use self::{
    bus::{AdminBus, ComposeBus, RefreshBus, RequestSender},
    composer::Composer,
    refresher::Refresher,
    router::Router,
    ticker::Ticker,
};

mod admin;
mod bus;
mod composer;
mod refresher;
mod router;
mod ticker;

const REFRESH_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Clone)]
struct ProxyState {
    admin_pathfinder_html: Html<String>,
    request_sender: RequestSender,
}

pub(super) async fn run(port: u16, expose: bool, config: FederatedGraphConfig) -> Result<(), crate::Error> {
    log::trace!("starting the federated dev server");

    let (graph_sender, graph_receiver) = mpsc::channel(16);
    let (refresh_sender, refresh_receiver) = mpsc::channel(16);
    let (compose_sender, compose_receiver) = mpsc::channel(16);
    let (request_sender, request_receiver) = mpsc::channel(16);

    let compose_bus = ComposeBus::new(graph_sender, refresh_sender, compose_sender.clone(), compose_receiver);
    let refresh_bus = RefreshBus::new(refresh_receiver, compose_sender.clone());
    let admin_bus = AdminBus::new(compose_sender.clone());

    let composer = Composer::new(compose_bus);
    tokio::spawn(composer.handler());

    let refresher = Refresher::new(refresh_bus);
    tokio::spawn(refresher.handler());

    let router = Router::new(graph_receiver, request_receiver, config);
    tokio::spawn(router.handler());

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
            request_sender,
        });

    let host = if expose {
        format!("0.0.0.0:{port}")
    } else {
        format!("127.0.0.1:{port}")
    };
    let address = host.parse().expect("we just defined it above, it _must work_");

    Server::bind(&address)
        .serve(app.into_make_service())
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
    State(ProxyState { request_sender, .. }): State<ProxyState>,
) -> impl IntoResponse {
    handle_engine_request(request, request_sender).await
}

async fn engine_post(
    State(ProxyState { request_sender, .. }): State<ProxyState>,
    Json(request): Json<engine::Request>,
) -> impl IntoResponse {
    handle_engine_request(request, request_sender).await
}

async fn handle_engine_request(request: engine::Request, request_sender: RequestSender) -> impl IntoResponse {
    let (response_sender, response_receiver) = oneshot::channel();
    request_sender.send((request, response_sender)).await.unwrap();

    match response_receiver.await {
        Ok(Ok(response)) => Json(response).into_response(),
        Ok(Err(error)) => Json(json!({
            "data": null,
            "errors": [
                {
                    "message": error.to_string(),
                    "locations": [],
                    "path": []
                }
            ]
        }))
        .into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response(),
    }
}
