use async_sse::Sender;
use bytes::Bytes;

use crate::utils::StreamJoinExt;
use futures::{AsyncBufReadExt, pin_mut};
use futures_util::{Stream, StreamExt, stream::BoxStream};
use headers::HeaderMapExt;

use crate::graphql_over_http::StreamingResponseFormat;

const MULTIPART_BOUNDARY: &str = "-";

pub fn encode_response<'a, T>(
    payload_stream: impl Stream<Item = T> + Send + 'a,
    streaming_format: StreamingResponseFormat,
) -> (http::HeaderMap, BoxStream<'a, Result<Bytes, String>>)
where
    T: serde::Serialize + Send,
{
    let bytes_stream: BoxStream<'a, Result<Bytes, String>> = match streaming_format {
        StreamingResponseFormat::IncrementalDelivery => {
            Box::pin(multipart_stream::serialize(
                payload_stream.map(|payload| {
                    let mut headers = http::HeaderMap::new();
                    headers.typed_insert(headers::ContentType::json());

                    Ok(multipart_stream::Part {
                        headers,
                        body: Bytes::from(sonic_rs::to_vec(&payload).map_err(|e| e.to_string())?),
                    })
                }),
                // The boundary we put in the header in execute_sreaming_request
                MULTIPART_BOUNDARY,
            ))
        }
        StreamingResponseFormat::GraphQLOverSSE => {
            let (sse_sender, sse_encoder) = async_sse::encode();

            let response_stream = sse_encoder.lines().map(|line| {
                line.map(|mut line| {
                    line.push_str("\r\n");
                    line.into()
                })
                .map_err(|e| e.to_string())
            });

            Box::pin(sse_stream(payload_stream, sse_sender, response_stream))
        }
        StreamingResponseFormat::GraphQLOverWebSocket => {
            unreachable!("Websocket response isn't returned as a HTTP response.")
        }
    };

    let mut headers = http::HeaderMap::new();

    headers.typed_insert(headers::CacheControl::new().with_no_cache());
    headers.typed_insert(headers::ContentType::from(match streaming_format {
        StreamingResponseFormat::IncrementalDelivery => format!("multipart/mixed; boundary=\"{MULTIPART_BOUNDARY}\"")
            .parse::<mime::Mime>()
            .expect("Valid Mime"),
        StreamingResponseFormat::GraphQLOverSSE => mime::TEXT_EVENT_STREAM,
        StreamingResponseFormat::GraphQLOverWebSocket => {
            unreachable!("Websocket response isn't returned as a HTTP response.")
        }
    }));

    (headers, bytes_stream)
}

fn sse_stream<'a, T>(
    payload_stream: impl Stream<Item = T> + Send + 'a,
    sse_sender: Sender,
    sse_output: impl Stream<Item = Result<Bytes, String>> + Send + 'a,
) -> impl Stream<Item = Result<Bytes, String>> + Send + 'a
where
    T: serde::Serialize + Send,
{
    // Pumps data from payload_stream into the sse_sender, and run that
    // alongside sse_output
    sse_output.join(async move {
        pin_mut!(payload_stream);

        while let Some(payload) = payload_stream.next().await {
            let payload_json = match sonic_rs::to_string(&payload) {
                Ok(json) => json,
                Err(error) => {
                    tracing::error!("Could not encode StreamingPayload as JSON: {error:?}");
                    return;
                }
            };

            if let Err(error) = sse_sender.send("next", &payload_json, None).await {
                tracing::error!("Could not send next payload via sse_sender: {error}");
                return;
            }
        }

        // The GraphQLOverSSE spec suggests we just need the event name on the complete
        // event but the SSE spec says that you should drop events with an empty data
        // buffer.  So I'm just putting null in the data buffer for now.
        if let Err(error) = sse_sender.send("complete", "null", None).await {
            tracing::error!("Could not send complete payload via sse_sender: {error}");
        }
    })
}
