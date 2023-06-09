use crate::{
    errors::ServerError,
    event::{wait_for_event, Event},
};
use axum::{
    extract::State,
    response::Html,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use std::net::{Ipv4Addr, SocketAddr};
use tower_http::trace::TraceLayer;

#[allow(clippy::unused_async)]
async fn playground(State(error): State<String>) -> Html<String> {
    let document = include_str!("error-page.html").replace("{{error}}", &error);

    Html(document)
}

#[allow(clippy::unused_async)]
async fn endpoint(State(error): State<String>) -> Json<Value> {
    let document = json!(
        {
            "data": null,
            "errors": [error]
        }
    );

    Json(document)
}

pub async fn start(
    port: u16,
    error: String,
    event_bus: tokio::sync::broadcast::Sender<Event>,
) -> Result<(), ServerError> {
    trace!("starting error server at port {port}");

    let router = Router::new()
        .route("/", get(playground))
        .route("/graphql", post(endpoint))
        .route("/graphql", get(endpoint))
        .with_state(error)
        .layer(TraceLayer::new_for_http());

    let socket_address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));

    let server = axum::Server::bind(&socket_address)
        .serve(router.into_make_service())
        .with_graceful_shutdown(wait_for_event(event_bus.subscribe(), |event| {
            event.should_restart_servers()
        }));

    server.await?;

    Ok(())
}
