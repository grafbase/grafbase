use axum::{extract::State, response::IntoResponse};
use futures_util::TryFutureExt;

use crate::{ServerRuntime, engine::into_axum_response, router::state::ServerState};

/// Executes a GraphQL request against the registered engine.
pub(crate) async fn execute<R: engine::Runtime, SR: ServerRuntime>(
    State(state): State<ServerState<R, SR>>,
    request: axum::extract::Request,
) -> impl IntoResponse {
    let engine = state.engine.borrow().clone();

    let (parts, body) = request.into_parts();
    let body = axum::body::to_bytes(body, state.request_body_limit_bytes).map_err(|error| {
        if let Some(source) = std::error::Error::source(&error)
            && source.is::<http_body_util::LengthLimitError>()
        {
            return (
                http::StatusCode::PAYLOAD_TOO_LARGE,
                format!("Request body exceeded: {}", state.request_body_limit_bytes),
            );
        }
        (http::StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
    });

    let response = into_axum_response(engine.execute(http::Request::from_parts(parts, body)).await);

    // lambda must flush the trace events here, otherwise the
    // function might fall asleep and the events are pending until
    // the next wake-up.
    //
    // read more: https://github.com/open-telemetry/opentelemetry-lambda/blob/main/docs/design_proposal.md
    #[cfg(feature = "lambda")]
    state.server_runtime.after_request();

    response
}
