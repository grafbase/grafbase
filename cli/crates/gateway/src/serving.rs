use axum::{
    extract::{RawQuery, State},
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use bytes::Bytes;
use engine::HttpGraphqlRequest;
use engine_v2_admin::AdminContext;
use http::HeaderMap;
use tower_http::cors::CorsLayer;

use crate::Gateway;

pub(super) fn router(gateway: Gateway) -> Router {
    Router::new()
        .route("/graphql", post(post_graphql).options(options_any).get(get_graphql))
        .route("/admin", post(post_admin_graphql).options(options_any))
        .with_state(gateway)
        .layer(CorsLayer::permissive())
        .layer(grafbase_tracing::tower::layer())
}

async fn post_admin_graphql(
    State(gateway): State<Gateway>,
    Json(request): Json<engine_v2_admin::Request>,
) -> impl IntoResponse {
    let response = engine_v2_admin::execute_admin_request(
        AdminContext {
            ray_id: ulid::Ulid::new().to_string(),
            cache: gateway.env.cache.clone(),
        },
        request,
    )
    .await;
    Json(response)
}

async fn post_graphql(State(gateway): State<Gateway>, headers: HeaderMap, body: Bytes) -> impl IntoResponse {
    gateway.execute(&headers, HttpGraphqlRequest::JsonBodyBytes(body)).await
}

async fn get_graphql(
    State(gateway): State<Gateway>,
    headers: HeaderMap,
    RawQuery(query): RawQuery,
) -> impl IntoResponse {
    gateway
        .execute(&headers, HttpGraphqlRequest::Query(query.unwrap_or_default().into()))
        .await
}

#[allow(clippy::unused_async)]
async fn options_any() -> impl IntoResponse {
    ""
}
