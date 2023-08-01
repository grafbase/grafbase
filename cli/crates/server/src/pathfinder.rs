#![allow(unused)]

use crate::{
    errors::ServerError,
    event::{wait_for_event, Event},
};
use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use common::environment::Environment;
use handlebars::Handlebars;
use hyper::{http::HeaderValue, Method};
use serde_json::json;
use std::net::{Ipv4Addr, SocketAddr, TcpListener};
use tower_http::{cors::CorsLayer, services::ServeDir};

pub async fn start(
    tcp_listener: TcpListener,
    port: u16,
    worker_port: u16,
    event_bus: tokio::sync::broadcast::Sender<Event>,
) -> Result<(), ServerError> {
    trace!("starting pathfinder at port {port}");

    let mut handlebars = Handlebars::new();
    let template = include_str!("../templates/pathfinder.hbs");
    handlebars
        .register_template_string("pathfinder.html", template)
        .expect("must be valid");
    let worker_url = format!("http://127.0.0.1:{worker_port}");
    let graphql_url = format!("{worker_url}/graphql");
    let asset_url = format!("http://127.0.0.1:{port}/static");
    let pathfinder_html = handlebars
        .render(
            "pathfinder.html",
            &json!({
                "ASSET_URL": asset_url,
                "GRAPHQL_URL": graphql_url
            }),
        )
        .expect("must render");

    let environment = Environment::get();
    let static_asset_path = environment.user_dot_grafbase_path.join("static");

    let router = Router::new()
        .route("/", get(root))
        .nest_service("/static", ServeDir::new(static_asset_path))
        .layer(
            CorsLayer::new()
                .allow_origin("*".parse::<HeaderValue>().expect("must parse"))
                .allow_methods([Method::GET]),
        )
        .with_state(Html(pathfinder_html));

    axum::Server::from_tcp(tcp_listener)
        .map_err(ServerError::StartPathfinderServer)?
        .serve(router.into_make_service())
        .with_graceful_shutdown(wait_for_event(event_bus.subscribe(), |event| {
            event.should_restart_servers()
        }))
        .await
        .map_err(ServerError::StartPathfinderServer);

    Ok(())
}

#[allow(clippy::unused_async)]
async fn root(State(pathfinder_html): State<Html<String>>) -> impl IntoResponse {
    pathfinder_html
}
