use axum::{
    extract::{Query, State},
    response::IntoResponse,
    routing::post,
    Router,
};
use bytes::Bytes;
use futures_util::future::{join_all, BoxFuture};
use gateway_core::StreamingFormat;
use http::{HeaderMap, StatusCode};
use tokio::sync::mpsc::{self, UnboundedReceiver};
use tower_http::cors::CorsLayer;

use crate::{Gateway, Response};

pub(super) fn router(gateway: Gateway) -> Router {
    Router::new()
        .route("/graphql", post(post_graphql).options(options_any).get(get_graphql))
        .with_state(gateway)
        .layer(CorsLayer::permissive())
        .layer(grafbase_tracing::tower::layer())
}

async fn post_graphql(State(gateway): State<Gateway>, headers: HeaderMap, body: Bytes) -> crate::Response {
    use gateway_core::ConstructableResponse as _;

    let streaming_format = headers
        .get(http::header::ACCEPT)
        .and_then(|value| value.to_str().ok())
        .and_then(StreamingFormat::from_accept_header);
    let (sender, receiver) = mpsc::unbounded_channel();
    let ctx = crate::Context::new(headers, sender);

    // FIXME: Pathfinder doesn't send the proper content-type, so axum complains about it.
    let request: engine::BatchRequest = match serde_json::from_slice(&body[..]) {
        Ok(req) => req,
        Err(err) => {
            return Response::error(StatusCode::BAD_REQUEST, &format!("Could not parse JSON request: {err}"));
        }
    };

    match request {
        engine::BatchRequest::Single(request) => {
            let result = match streaming_format {
                Some(format) => gateway.execute_stream(&ctx, request, format).await,
                None => gateway
                    .execute(&ctx, request)
                    .await
                    .and_then(|(response, headers)| Response::engine(response, headers)),
            };

            let response = match result {
                Ok(response) => response,
                Err(error) => crate::Response::from(error),
            };

            tokio::spawn(wait(receiver));
            response
        }
        engine::BatchRequest::Batch(requests) => {
            if streaming_format.is_some() {
                return crate::Response::error(
                    StatusCode::BAD_REQUEST,
                    "batch requests can't use multipart or event-stream responses",
                );
            }
            let mut responses = Vec::with_capacity(requests.len());
            for request in requests {
                responses.push(match gateway.execute(&ctx, request).await {
                    Ok((response, _)) => response,
                    Err(error) => return crate::Response::from(error),
                });
            }

            crate::Response::batch_response(responses)
        }
    }
}

async fn get_graphql(
    State(gateway): State<Gateway>,
    headers: HeaderMap,
    Query(params): Query<engine::QueryParamRequest>,
) -> crate::Response {
    use gateway_core::ConstructableResponse as _;

    let streaming_format = headers
        .get(http::header::ACCEPT)
        .and_then(|value| value.to_str().ok())
        .and_then(StreamingFormat::from_accept_header);
    let (sender, receiver) = mpsc::unbounded_channel();
    let ctx = crate::Context::new(headers, sender);

    let mut request: engine::Request = params.into();
    request.ray_id = ctx.ray_id.clone();

    let result = match streaming_format {
        None => gateway
            .execute(&ctx, request)
            .await
            .and_then(|(response, headers)| Response::engine(response, headers)),
        Some(streaming_format) => gateway.execute_stream(&ctx, request, streaming_format).await,
    };

    let response = match result {
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
