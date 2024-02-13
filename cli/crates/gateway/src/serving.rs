use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    response::IntoResponse,
    routing::post,
    Router,
};
use bytes::Bytes;
use futures_util::future::{join_all, BoxFuture};
use gateway_core::{
    serving::{AUTHORIZATION_HEADER, X_API_KEY_HEADER},
    StreamingFormat,
};
use http::{HeaderMap, StatusCode};
use tokio::sync::mpsc::{self, UnboundedReceiver};
use tower_http::cors::CorsLayer;

use crate::{Gateway, Response};

pub(super) fn router(gateway: Gateway) -> Router {
    Router::new()
        .route("/graphql", post(post_graphql).options(options_any).get(get_graphql))
        .with_state(gateway)
        .layer(CorsLayer::permissive())
}

async fn post_graphql(
    State(gateway): State<Gateway>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
    body: Bytes,
) -> crate::Response {
    use gateway_core::ConstructableResponse as _;

    let streaming_format = headers
        .get(http::header::ACCEPT)
        .and_then(|value| value.to_str().ok())
        .and_then(StreamingFormat::from_accept_header);
    let (sender, receiver) = mpsc::unbounded_channel();
    let ctx = crate::Context::new(headers, &params, sender);
    // FIXME: Pathfinder doesn't send the proper content-type, so axum complains about it.
    let request: engine::Request = match serde_json::from_slice(&body[..]) {
        Ok(req) => req,
        Err(err) => {
            return Response::error(StatusCode::BAD_REQUEST, &format!("Could not parse JSON request: {err}"));
        }
    };

    let response = match gateway.execute(&ctx, request, streaming_format).await {
        Ok(response) => response,
        Err(error) => crate::Response::from(error),
    };

    tokio::spawn(wait(receiver));
    response
}

#[derive(serde::Deserialize)]
struct GetRequestParams {
    #[serde(flatten)]
    request: engine::QueryParamRequest,
    #[serde(default, rename = "x-api-key")]
    x_api_key: Option<String>,
    #[serde(default)]
    authorization: Option<String>,
}

impl GetRequestParams {
    fn auth_query_params(&self) -> HashMap<String, String> {
        [
            self.x_api_key.clone().map(|key| (X_API_KEY_HEADER.to_string(), key)),
            self.authorization
                .clone()
                .map(|auth| (AUTHORIZATION_HEADER.to_string(), auth)),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

async fn get_graphql(
    State(gateway): State<Gateway>,
    headers: HeaderMap,
    Query(params): Query<GetRequestParams>,
) -> crate::Response {
    let streaming_format = headers
        .get(http::header::ACCEPT)
        .and_then(|value| value.to_str().ok())
        .and_then(StreamingFormat::from_accept_header);
    let (sender, receiver) = mpsc::unbounded_channel();
    let ctx = crate::Context::new(headers, &params.auth_query_params(), sender);

    let mut request: engine::Request = params.request.into();
    request.ray_id = ctx.ray_id.clone();
    let response = match gateway.execute(&ctx, request, streaming_format).await {
        Ok(response) => response,
        Err(error) => Response::from(error),
    };

    tokio::spawn(wait(receiver));
    response
}

#[allow(clippy::unused_async)]
async fn options_any() -> impl IntoResponse {
    ""
}

async fn wait(mut receiver: UnboundedReceiver<BoxFuture<'static, ()>>) {
    // Wait simultaneously on everything immediately accessible
    join_all(std::iter::from_fn(|| receiver.try_recv().ok())).await;
    // Wait sequentially on the rest
    while let Some(fut) = receiver.recv().await {
        fut.await;
    }
}
