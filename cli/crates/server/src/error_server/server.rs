use crate::errors::ServerError;
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use std::net::Ipv4Addr;
use tokio_util::sync::CancellationToken;
use tower_http::trace::TraceLayer;

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

pub async fn start(port: u16, error: String, cancel_token: CancellationToken) -> Result<(), ServerError> {
    trace!("starting error server at port {port}");

    let router = Router::new()
        .route("/graphql", post(endpoint))
        .route("/graphql", get(endpoint))
        .with_state(error)
        .layer(TraceLayer::new_for_http());

    let server = axum::Server::bind(&std::net::SocketAddr::from((Ipv4Addr::LOCALHOST, port)))
        .serve(router.into_make_service())
        .with_graceful_shutdown(cancel_token.cancelled());

    server.await?;

    Ok(())
}
