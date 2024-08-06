use super::ServerState;
use axum::{extract::State, response::IntoResponse};

pub(super) async fn execute(State(state): State<ServerState>, request: axum::extract::Request) -> impl IntoResponse {
    let Some(engine) = state.gateway().borrow().clone() else {
        return engine_v2_axum::internal_server_error("there are no subgraphs registered currently");
    };

    let response = engine_v2_axum::execute(engine, request, state.request_body_limit_bytes()).await;

    // lambda must flush the trace events here, otherwise the
    // function might fall asleep and the events are pending until
    // the next wake-up.
    //
    // read more: https://github.com/open-telemetry/opentelemetry-lambda/blob/main/docs/design_proposal.md
    #[cfg(feature = "lambda")]
    if let Some(provider) = state.tracer_provider() {
        for result in provider.force_flush() {
            if let Err(e) = result {
                println!("error flushing events: {e}");
            }
        }
    }

    response
}
