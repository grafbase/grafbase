use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, routing::get, Router};
use engine_v2::Engine;

use super::TestRuntime;

pub(super) fn build(engine: Arc<Engine<TestRuntime>>) -> Router {
    Router::new()
        .route("/graphql", get(execute).post(execute))
        .with_state(engine)
}

async fn execute(State(engine): State<Arc<Engine<TestRuntime>>>, request: axum::extract::Request) -> impl IntoResponse {
    engine_v2_axum::execute(engine, request, usize::MAX).await
}
