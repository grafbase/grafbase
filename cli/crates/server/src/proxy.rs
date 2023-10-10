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
use std::sync::Arc;
use std::time::Duration;
use std::{
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, TcpListener},
    sync::atomic::Ordering,
};
use tokio::signal;
use tokio::time::sleep;
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
        .http1_preserve_header_case(true)
        .serve(router.into_make_service())
        .await
        .map_err(ServerError::StartProxyServer);

    Ok(())
}

#[allow(clippy::unused_async)]
async fn root(State(ProxyState { pathfinder_html, .. }): State<ProxyState>) -> impl IntoResponse {
    pathfinder_html
}

const POLL_INTERVAL: Duration = Duration::from_millis(200);

async fn graphql(
    State(ProxyState { client, .. }): State<ProxyState>,
    mut req: Request<Body>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let query = req.uri().query().map_or(String::new(), |query| format!("?{query}"));

    // http::Request can't be cloned
    let (parts, body) = req.into_parts();
    let body_bytes = hyper::body::to_bytes(body).await.map_err(|_| StatusCode::BAD_REQUEST)?;
    let body = String::from_utf8(body_bytes.clone().into()).map_err(|_| StatusCode::BAD_REQUEST)?;
    req = Request::from_parts(parts, body_bytes.into());

    loop {
        let worker_port = WORKER_PORT.load(Ordering::Relaxed);

        if worker_port == 0 {
            sleep(POLL_INTERVAL).await;
            continue;
        }

        let uri = format!("http://127.0.0.1:{worker_port}/graphql{query}");

        // http::Request can't be cloned
        let mut cloned_request = Request::builder()
            .method(req.method().clone())
            .uri(uri)
            .version(req.version());

        for header in req.headers() {
            cloned_request = cloned_request.header(header.0, header.1);
        }

        let request = cloned_request
            .body(body.clone().into())
            .expect("must succeed, using an existing request");

        let response = client.request(request).await;

        match response {
            Ok(response) => {
                return Ok(response);
            }
            Err(error) => {
                if error.is_connect() {
                    sleep(POLL_INTERVAL).await;
                } else {
                    return Err(StatusCode::BAD_REQUEST);
                }
            }
        };
    }
}
