pub(super) mod format;

use async_sse::Sender;
use bytes::Bytes;
use format::StreamingFormat;
use futures_util::{
    future::{self, BoxFuture},
    pin_mut,
    stream::{self, BoxStream},
    AsyncBufReadExt, FutureExt, Stream, StreamExt,
};
use headers::HeaderMapExt;

const MULTIPART_BOUNDARY: &str = "-";

pub async fn encode_stream_response<'a, T>(
    ray_id: String,
    payload_stream: impl Stream<Item = T> + Send + 'a,
    streaming_format: StreamingFormat,
) -> (http::HeaderMap, BoxStream<'a, Result<Bytes, String>>)
where
    T: serde::Serialize + Send,
{
    let bytes_stream: BoxStream<'a, Result<Bytes, String>> = match streaming_format {
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
                // The boundary we put in the header in execute_sreaming_request
                MULTIPART_BOUNDARY,
            ))
        }
        StreamingFormat::GraphQLOverSSE => {
            let (sse_sender, sse_encoder) = async_sse::encode();
            let response_stream: BoxStream<'a, Result<Bytes, String>> = Box::pin(sse_encoder.lines().map(|line| {
                line.map(|mut line| {
                    line.push_str("\r\n");
                    line.into()
                })
                .map_err(|e| e.to_string())
            }));

            sse_stream(ray_id, payload_stream, sse_sender, response_stream)
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
    sse_output: BoxStream<'a, Result<Bytes, String>>,
) -> BoxStream<'a, Result<Bytes, String>>
where
    T: serde::Serialize + Send,
{
    // Start a future that pumps data from payload_stream into the sse_sender
    let pump_future: BoxFuture<'a, ()> = Box::pin(async move {
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
    });

    // Return a Stream that'll run `pump_future` while taking items from sse_output
    Box::pin(futures_util::stream::unfold(
        SSEStreamState::Running(sse_output.fuse(), pump_future.fuse()),
        |mut state| async {
            loop {
                match state {
                    SSEStreamState::Running(mut sse_output, mut pump) => {
                        futures_util::select! {
                            output = sse_output.next() => {
                                return Some((output?, SSEStreamState::Running(sse_output, pump)));
                            }
                            _ = pump => {
                                state = SSEStreamState::Draining(sse_output);
                                continue;
                            }
                        }
                    }
                    SSEStreamState::Draining(mut sse_output) => {
                        return Some((sse_output.next().await?, SSEStreamState::Draining(sse_output)))
                    }
                }
            }
        },
    ))
}

enum SSEStreamState<'a> {
    Running(
        stream::Fuse<BoxStream<'a, Result<Bytes, String>>>,
        future::Fuse<BoxFuture<'a, ()>>,
    ),
    Draining(stream::Fuse<BoxStream<'a, Result<Bytes, String>>>),
}
