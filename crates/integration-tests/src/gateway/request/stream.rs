use std::future::IntoFuture;

use futures::{StreamExt, TryStreamExt, future::BoxFuture, stream::BoxStream};
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
            let stream = multipart_stream::parse(body.into_data_stream(), "-")
                .map(|result| match result {
                    Ok(part) => match serde_json::from_slice(&part.body) {
                        Ok(value) => value,
                        Err(error) => serde_json::Value::String(format!("JSON serialization error: {error}")),
                    },
                    Err(error) => serde_json::Value::String(format!("Multipart error: {error}")),
                })
                .inspect(|value| {
                    tracing::debug!("Weboscket event:\n{}", serde_json::to_string_pretty(&value).unwrap());
                })
                .boxed();
            GraphqlStreamingResponse {
                status: parts.status,
                headers: parts.headers,
                stream,
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
            let stream = body.into_data_stream().map_err(std::io::Error::other);
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
                })
                .inspect(|value| {
                    tracing::debug!("SSE event:\n{}", serde_json::to_string_pretty(&value).unwrap());
                })
                .boxed();
            GraphqlStreamingResponse {
                status: parts.status,
                headers: parts.headers,
                stream,
            }
        })
    }
}

pub struct GraphqlStreamingResponse {
    pub status: http::StatusCode,
    pub headers: http::HeaderMap,
    pub stream: BoxStream<'static, serde_json::Value>,
}

impl GraphqlStreamingResponse {
    pub async fn next(&mut self) -> Option<serde_json::Value> {
        self.stream.next().await
    }

    pub async fn collect(self) -> GraphqlCollectedStreamingResponse {
        let messages = self.stream.collect().await;
        GraphqlCollectedStreamingResponse {
            status: self.status,
            headers: self.headers,
            messages,
        }
    }
}

pub struct GraphqlCollectedStreamingResponse {
    pub status: http::StatusCode,
    pub headers: http::HeaderMap,
    pub messages: Vec<serde_json::Value>,
}
