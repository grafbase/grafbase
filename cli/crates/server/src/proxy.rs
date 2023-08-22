#![allow(unused)]

use crate::atomics::WORKER_PORT;
use crate::{
    errors::ServerError,
    event::{wait_for_event, Event},
};
use axum::routing::head;
use axum::{
    body::{Body, HttpBody},
    extract::{Query, RawPathParams, State},
    http::uri::Uri,
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
    Router,
};
use common::environment::Environment;
use handlebars::Handlebars;
use hyper::{client::HttpConnector, StatusCode};
use hyper::{http::HeaderValue, Method, Request};
use serde_json::json;
use sqlx::query;
use std::net::Shutdown;
use std::{
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, TcpListener},
    sync::atomic::Ordering,
};
use tokio::signal;
use tower_http::{cors::CorsLayer, services::ServeDir};

type Client = hyper::client::Client<HttpConnector, Body>;

#[derive(Clone)]
struct ProxyState {
    pathfinder_html: Html<String>,
    client: Client,
}

pub async fn start(listener: TcpListener, event_bus: tokio::sync::broadcast::Sender<Event>) -> Result<(), ServerError> {
    let port = listener.local_addr().expect("must have a local addr").port();
    trace!("starting pathfinder at port {port}");

    let client: Client = hyper::Client::builder()
        .http1_preserve_header_case(true)
        .build(HttpConnector::new());

    let mut handlebars = Handlebars::new();
    let template = include_str!("../templates/pathfinder.hbs");
    handlebars
        .register_template_string("pathfinder.html", template)
        .expect("must be valid");
    let proxied_graphql_url = "/graphql".to_string();
    let asset_url = format!("http://127.0.0.1:{port}/static");
    let pathfinder_html = handlebars
        .render(
            "pathfinder.html",
            &json!({
                "ASSET_URL": asset_url,
                "GRAPHQL_URL": proxied_graphql_url
            }),
        )
        .expect("must render");

    let environment = Environment::get();
    let static_asset_path = environment.user_dot_grafbase_path.join("static");

    let router = Router::new()
        .route("/", get(root))
        .route("/graphql", get(graphql))
        .route("/graphql", head(graphql))
        .route("/graphql", post(graphql))
        .nest_service("/static", ServeDir::new(static_asset_path))
        .layer(CorsLayer::permissive())
        .with_state(ProxyState {
            pathfinder_html: Html(pathfinder_html),
            client,
        });

    axum::Server::from_tcp(listener)
        .map_err(ServerError::StartProxyServer)?
        .http1_title_case_headers(true)
        .serve(router.into_make_service())
        .await
        .map_err(ServerError::StartProxyServer);

    Ok(())
}

#[allow(clippy::unused_async)]
async fn root(State(ProxyState { pathfinder_html, .. }): State<ProxyState>) -> impl IntoResponse {
    pathfinder_html
}

async fn graphql(
    State(ProxyState { client, .. }): State<ProxyState>,
    mut req: Request<Body>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let query = req.uri().query().map_or(String::new(), |query| format!("?{query}"));

    let worker_port = WORKER_PORT.load(Ordering::Relaxed);

    dbg!(worker_port);

    if worker_port == 0 {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }

    let uri = format!("http://127.0.0.1:{worker_port}/graphql{query}");

    *req.uri_mut() = Uri::try_from(uri).expect("must be valid");

    let response = client.request(req).await;

    dbg!(&response);

    match response {
        Ok(response) => Ok(response),
        Err(error) => {
            if error.is_connect() {
                Err(StatusCode::SERVICE_UNAVAILABLE)
            } else {
                Err(StatusCode::BAD_REQUEST)
            }
        }
    }
}
