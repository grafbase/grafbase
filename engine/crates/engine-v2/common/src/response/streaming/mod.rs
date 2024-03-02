mod format;

use async_runtime::stream::StreamExt as _;
use async_sse::Sender;
use bytes::Bytes;
pub use format::*;
use futures_util::{pin_mut, stream::BoxStream, AsyncBufReadExt, Stream, StreamExt, TryStreamExt};
use headers::HeaderMapExt;
use runtime::bytes::OwnedOrSharedBytes;

const MULTIPART_BOUNDARY: &str = "-";

pub(super) async fn encode_stream_response<'a, T>(
    ray_id: String,
    payload_stream: impl Stream<Item = T> + Send + 'a,
    streaming_format: StreamingFormat,
) -> (http::HeaderMap, BoxStream<'a, Result<OwnedOrSharedBytes, String>>)
where
    T: serde::Serialize + Send,
{
    let bytes_stream: BoxStream<'a, Result<OwnedOrSharedBytes, String>> = match streaming_format {
        StreamingFormat::IncrementalDelivery => {
            Box::pin(
                multipart_stream::serialize(
                    payload_stream.map(|payload| {
                        let mut headers = http::HeaderMap::new();
                        headers.typed_insert(headers::ContentType::json());
                        Ok(multipart_stream::Part {
                            headers,
                            body: Bytes::from(serde_json::to_vec(&payload).map_err(|e| e.to_string())?),
                        })
                    }),
                    // The boundary we put in the header in execute_sreaming_request
                    MULTIPART_BOUNDARY,
                )
                .map_ok(Into::into),
            )
        }
        StreamingFormat::GraphQLOverSSE => {
            let (sse_sender, sse_encoder) = async_sse::encode();
            let response_stream = sse_encoder.lines().map(|line| {
                line.map(|mut line| {
                    line.push_str("\r\n");
                    line.into()
                })
                .map_err(|e| e.to_string())
            });

            Box::pin(sse_stream(ray_id, payload_stream, sse_sender, response_stream))
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

fn sse_stream<'a, T>(
    ray_id: String,
    payload_stream: impl Stream<Item = T> + Send + 'a,
    sse_sender: Sender,
    sse_output: impl Stream<Item = Result<OwnedOrSharedBytes, String>> + Send + 'a,
) -> impl Stream<Item = Result<OwnedOrSharedBytes, String>> + Send + 'a
where
    T: serde::Serialize + Send,
{
    // Pumps data from payload_stream into the sse_sender, and run that
    // alongside sse_output
    sse_output.join(async move {
        pin_mut!(payload_stream);

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
    })
}
