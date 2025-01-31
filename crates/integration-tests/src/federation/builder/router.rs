use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, routing::get, Router};
use engine::Engine;
use engine_axum::websocket::{WebsocketAccepter, WebsocketService};
use gateway_config::Config;

use super::TestRuntime;

pub(super) fn build(engine: Arc<Engine<TestRuntime>>, config: &Config) -> Router {
    let (websocket_sender, websocket_receiver) = tokio::sync::mpsc::channel(16);
    let (_, engine_watcher) = tokio::sync::watch::channel(engine.clone());
    let websocket_accepter = WebsocketAccepter::new(websocket_receiver, engine_watcher);
    tokio::spawn(websocket_accepter.handler());

    let graphql_path = config.graph.path.as_deref().unwrap_or("/graphql");
    let websocket_path = config.graph.websocket_path.as_deref().unwrap_or("/ws");

    Router::new()
        .route(graphql_path, get(execute).post(execute))
        .route_service(websocket_path, WebsocketService::new(websocket_sender))
        .with_state(engine)
}

async fn execute(State(engine): State<Arc<Engine<TestRuntime>>>, request: axum::extract::Request) -> impl IntoResponse {
    engine_axum::execute(engine, request, usize::MAX).await
}
