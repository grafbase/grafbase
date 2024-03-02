use super::{gateway::EngineWatcher, ServerState};
use axum::{
    body::{Body, Bytes},
    extract::{RawQuery, State},
    response::IntoResponse,
    Json,
};
use engine_v2::HttpGraphqlRequest;
use http::HeaderMap;
use serde_json::json;

pub(super) async fn get(
    headers: HeaderMap,
    RawQuery(query): RawQuery,
    State(ServerState { gateway, .. }): State<ServerState>,
) -> impl IntoResponse {
    handle(
        headers,
        HttpGraphqlRequest::Query(query.unwrap_or_default().into()),
        gateway,
    )
    .await
}

pub(super) async fn post(
    State(ServerState { gateway, .. }): State<ServerState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    handle(headers, HttpGraphqlRequest::JsonBodyBytes(body), gateway).await
}

async fn handle(headers: HeaderMap, request: HttpGraphqlRequest<'_>, engine: EngineWatcher) -> http::Response<Body> {
    let Some(engine) = engine.borrow().clone() else {
        return Json(json!({
            "errors": [{"message": "there are no subgraphs registered currently"}]
        }))
        .into_response();
    };
    let ray_id = ulid::Ulid::new().to_string();
    engine.execute(headers, &ray_id, request).await.into_response()
}
