use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, routing::get, Router};
use engine::Engine;
use engine_axum::websocket::{WebsocketAccepter, WebsocketService};

use super::TestRuntime;

pub(super) fn build(engine: Arc<Engine<TestRuntime>>) -> Router {
    let (websocket_sender, websocket_receiver) = tokio::sync::mpsc::channel(16);
    let (_, engine_watcher) = tokio::sync::watch::channel(engine.clone());
    let websocket_accepter = WebsocketAccepter::new(websocket_receiver, engine_watcher);
    tokio::spawn(websocket_accepter.handler());

    Router::new()
        .route("/graphql", get(execute).post(execute))
        .route_service("/ws", WebsocketService::new(websocket_sender))
        .with_state(engine)
}

async fn execute(State(engine): State<Arc<Engine<TestRuntime>>>, request: axum::extract::Request) -> impl IntoResponse {
    engine_axum::execute(engine, request, usize::MAX).await
}
