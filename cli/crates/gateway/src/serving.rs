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
    serving::{OPERATION_NAME_REQUEST_PARAMETER, QUERY_REQUEST_PARAMETER, VARIABLES_REQUEST_PARAMETER},
    StreamingFormat,
};
use http::{HeaderMap, StatusCode};
use tokio::sync::mpsc::{self, UnboundedReceiver};
use tower_http::cors::CorsLayer;

use crate::{Error, Gateway, Response};

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

async fn get_graphql(
    State(gateway): State<Gateway>,
    headers: HeaderMap,
    Query(mut params): Query<HashMap<String, String>>,
) -> crate::Response {
    let streaming_format = headers
        .get(http::header::ACCEPT)
        .and_then(|value| value.to_str().ok())
        .and_then(StreamingFormat::from_accept_header);
    let (sender, receiver) = mpsc::unbounded_channel();
    let ctx = crate::Context::new(headers, &params, sender);

    let Some(query) = params.remove(QUERY_REQUEST_PARAMETER) else {
        return Error::BadRequest("Missing 'query' parameter".into()).into();
    };

    let request = engine::Request::new(query)
        .with_operation_name(params.remove(OPERATION_NAME_REQUEST_PARAMETER).unwrap_or_default())
        .variables(
            params
                .get(VARIABLES_REQUEST_PARAMETER)
                .and_then(|variables| serde_json::from_str(variables).ok())
                .unwrap_or_default(),
        );

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
