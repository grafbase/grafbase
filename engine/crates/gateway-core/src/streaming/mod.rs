pub(super) mod format;

use bytes::Bytes;
use engine::StreamingPayload;
use format::StreamingFormat;
use futures_util::{stream::BoxStream, AsyncBufReadExt, Stream, StreamExt};
use headers::HeaderMapExt;

const MULTIPART_BOUNDARY: &str = "-";

pub async fn encode_stream_response(
    ctx: &impl crate::RequestContext,
    payload_stream: impl Stream<Item = StreamingPayload> + Send + 'static,
    streaming_format: StreamingFormat,
) -> (http::HeaderMap, BoxStream<'static, Result<Bytes, String>>) {
    let bytes_stream: BoxStream<'static, Result<Bytes, String>> = match streaming_format {
        StreamingFormat::IncrementalDelivery => {
            Box::pin(multipart_stream::serialize(
                payload_stream.map(|payload| {
                    let mut headers = http::HeaderMap::new();
                    headers.typed_insert(headers::ContentType::json());
                    Ok(multipart_stream::Part {
                        headers,
                        body: Bytes::from(serde_json::to_vec(&payload).map_err(|e| e.to_string())?),
                    })
                }),
                // The boundary we put in the header in execute_streaming_request
                MULTIPART_BOUNDARY,
            ))
        }
        StreamingFormat::GraphQLOverSSE => {
            let mut payload_stream = Box::pin(payload_stream);

            let (sse_sender, sse_encoder) = async_sse::encode();
            let response_stream = sse_encoder.lines().map(|line| {
                line.map(|mut line| {
                    line.push_str("\r\n");
                    line.into()
                })
                .map_err(|e| e.to_string())
            });

            let ray_id = ctx.ray_id().to_string();
            ctx.wait_until(Box::pin(async move {
                while let Some(payload) = payload_stream.next().await {
                    let payload_json = match serde_json::to_string(&payload) {
                        Ok(json) => json,
                        Err(error) => {
                            log::error!(ray_id, "Could not encode StreamingPayload as JSON: {error:?}");
                            return;
                        }
                    };

                    if let Err(error) = sse_sender.send("next", &payload_json, None).await {
                        log::error!(ray_id, "Could not send next payload via sse_sender: {error}");
                        return;
                    }
                }

                // The GraphQLOverSSE spec suggests we just need the event name on the complete
                // event but the SSE spec says that you should drop events with an empty data
                // buffer.  So I'm just putting null in the data buffer for now.
                if let Err(error) = sse_sender.send("complete", "null", None).await {
                    log::error!(ray_id, "Could not send complete payload via sse_sender: {error}");
                }
            }))
            .await;

            Box::pin(response_stream)
        }
    };

    let mut headers = http::HeaderMap::new();
    headers.typed_insert(headers::CacheControl::new().with_no_cache());
    headers.typed_insert(headers::ContentType::from(match streaming_format {
        StreamingFormat::IncrementalDelivery => format!("multipart/mixed; boundary=\"{MULTIPART_BOUNDARY}\"")
            .parse::<mime::Mime>()
            .expect("Valid Mime"),
        StreamingFormat::GraphQLOverSSE => mime::TEXT_EVENT_STREAM,
    }));

    (headers, bytes_stream)
}
