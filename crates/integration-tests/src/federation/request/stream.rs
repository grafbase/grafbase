use std::future::IntoFuture;

use futures::{future::BoxFuture, StreamExt, TryStreamExt};
use tower::ServiceExt;

pub struct MultipartStreamRequest(pub(super) super::TestRequest);

impl IntoFuture for MultipartStreamRequest {
    type Output = GraphqlStreamingResponse;

    type IntoFuture = BoxFuture<'static, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        let (router, mut request) = self.0.into_router_and_request();
        request
            .headers_mut()
            .entry(http::header::ACCEPT)
            .or_insert(http::HeaderValue::from_static("multipart/mixed,application/json;q=0.9"));
        Box::pin(async move {
            let (parts, body) = router.oneshot(request).await.unwrap().into_parts();
            let stream = multipart_stream::parse(body.into_data_stream(), "-").map(|result| match result {
                Ok(part) => match serde_json::from_slice(&part.body) {
                    Ok(value) => value,
                    Err(error) => serde_json::Value::String(format!("JSON serialization error: {error}")),
                },
                Err(error) => serde_json::Value::String(format!("Multipart error: {error}")),
            });
            GraphqlStreamingResponse {
                status: parts.status,
                headers: parts.headers,
                collected_body: stream.collect().await,
            }
        })
    }
}

pub struct SseStreamRequest(pub(super) super::TestRequest);

impl IntoFuture for SseStreamRequest {
    type Output = GraphqlStreamingResponse;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let (router, mut request) = self.0.into_router_and_request();
        request
            .headers_mut()
            .entry(http::header::ACCEPT)
            .or_insert(http::HeaderValue::from_static("text/event-stream"));
        Box::pin(async move {
            let (parts, body) = router.oneshot(request).await.unwrap().into_parts();
            let stream = body
                .into_data_stream()
                .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err));
            let stream = async_sse::decode(stream.into_async_read())
                .into_stream()
                .try_take_while(|event| {
                    let take = if let async_sse::Event::Message(msg) = event {
                        msg.name() != "complete"
                    } else {
                        false
                    };
                    futures::future::ready(Ok(take))
                })
                .map(|result| match result {
                    Ok(async_sse::Event::Retry(_)) => serde_json::Value::String("Got retry?".into()),
                    Ok(async_sse::Event::Message(msg)) => serde_json::from_slice(msg.data()).unwrap(),
                    Err(err) => serde_json::Value::String(err.to_string()),
                });
            GraphqlStreamingResponse {
                status: parts.status,
                headers: parts.headers,
                collected_body: stream.collect().await,
            }
        })
    }
}

pub struct GraphqlStreamingResponse {
    pub status: http::StatusCode,
    pub headers: http::HeaderMap,
    pub collected_body: Vec<serde_json::Value>,
}
