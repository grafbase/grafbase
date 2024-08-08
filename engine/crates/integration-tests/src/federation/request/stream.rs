use std::future::IntoFuture;

use bytes::Bytes;
use engine::BatchRequest;
use futures::{future::BoxFuture, stream::BoxStream, StreamExt, TryStreamExt};
use gateway_core::StreamingFormat;
use headers::HeaderMapExt;

pub struct MultipartStreamRequest(pub(super) super::ExecutionRequest);

impl MultipartStreamRequest {
    pub async fn collect<B>(self) -> B
    where
        B: Default + Extend<serde_json::Value>,
    {
        self.await.stream.collect().await
    }
}

impl IntoFuture for MultipartStreamRequest {
    type Output = GraphqlStreamingResponse;

    type IntoFuture = BoxFuture<'static, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        let mut headers = self.0.http_headers();
        headers.typed_insert(StreamingFormat::IncrementalDelivery);
        let request = BatchRequest::Single(self.0.request.into_engine_request());
        Box::pin(async move {
            let response = self.0.engine.execute(headers, request).await;
            let stream = multipart_stream::parse(response.body.into_stream().map_ok(Into::into), "-")
                .map(|result| serde_json::from_slice(&result.unwrap().body).unwrap());
            GraphqlStreamingResponse {
                headers: response.headers,
                stream: Box::pin(stream),
            }
        })
    }
}

pub struct SseStreamRequest(pub(super) super::ExecutionRequest);

impl SseStreamRequest {
    pub async fn collect<B>(self) -> B
    where
        B: Default + Extend<serde_json::Value>,
    {
        self.await.stream.collect().await
    }
}

impl IntoFuture for SseStreamRequest {
    type Output = GraphqlStreamingResponse;
    type IntoFuture = BoxFuture<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let mut headers = self.0.http_headers();
        headers.typed_insert(StreamingFormat::GraphQLOverSSE);
        let request = BatchRequest::Single(self.0.request.into_engine_request());
        Box::pin(async move {
            let response = self.0.engine.execute(headers, request).await;
            let stream = response.body.into_stream().map(|result| match result {
                Ok(bytes) => Ok(Bytes::from(bytes)),
                Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
            });
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
                headers: response.headers,
                stream: Box::pin(stream),
            }
        })
    }
}

pub struct GraphqlStreamingResponse {
    pub headers: http::HeaderMap,
    pub stream: BoxStream<'static, serde_json::Value>,
}
