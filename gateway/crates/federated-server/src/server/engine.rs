use super::{gateway::EngineWatcher, ServerState};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use engine::BatchRequest;
use grafbase_tracing::otel::opentelemetry_sdk::trace::TracerProvider;
use http::HeaderMap;

pub(super) async fn get(
    Query(request): Query<engine::QueryParamRequest>,
    headers: HeaderMap,
    State(state): State<ServerState>,
) -> impl IntoResponse {
    let request = engine::BatchRequest::Single(request.into());
    traced(headers, request, state.gateway().clone(), state.tracer_provider()).await
}

pub(super) async fn post(
    State(state): State<ServerState>,
    headers: HeaderMap,
    Json(request): Json<engine::BatchRequest>,
) -> impl IntoResponse {
    traced(headers, request, state.gateway().clone(), state.tracer_provider()).await
}

#[cfg(feature = "lambda")]
async fn traced(
    headers: HeaderMap,
    request: BatchRequest,
    engine: EngineWatcher,
    provider: Option<TracerProvider>,
) -> impl IntoResponse {
    let response = handle(headers, request, engine).await;

    // lambda must flush the trace events here, otherwise the
    // function might fall asleep and the events are pending until
    // the next wake-up.
    //
    // read more: https://github.com/open-telemetry/opentelemetry-lambda/blob/main/docs/design_proposal.md
    if let Some(provider) = provider {
        for result in provider.force_flush() {
            if let Err(e) = result {
                println!("error flushing events: {e}");
            }
        }
    }

    response
}

#[cfg(not(feature = "lambda"))]
async fn traced(
    headers: HeaderMap,
    request: BatchRequest,
    engine: EngineWatcher,
    _: Option<TracerProvider>,
) -> impl IntoResponse {
    handle(headers, request, engine).await
}

async fn handle(headers: HeaderMap, request: BatchRequest, engine: EngineWatcher) -> impl IntoResponse {
    let Some(engine) = engine.borrow().clone() else {
        return engine_v2_axum::internal_server_error("there are no subgraphs registered currently");
    };
    engine_v2_axum::into_response(engine.execute(headers, request).await)
}
