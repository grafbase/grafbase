use std::future::Future;

use bytes::Bytes;
use futures_util::Stream;
use futures_util::{StreamExt, TryStreamExt};
use reqwest::RequestBuilder;
use reqwest_eventsource::RequestBuilderExt;
use runtime::bytes::OwnedOrSharedBytes;
use runtime::fetch::{FetchError, FetchRequest, FetchResult, Fetcher};
use runtime::hooks::ResponseInfo;

#[derive(Default, Clone)]
pub struct NativeFetcher {
    client: reqwest::Client,
}

impl Fetcher for NativeFetcher {
    async fn fetch(
        &self,
        request: FetchRequest<'_, Bytes>,
    ) -> (FetchResult<http::Response<OwnedOrSharedBytes>>, Option<ResponseInfo>) {
        let mut info = ResponseInfo::builder();

        let result = self.client.execute(into_reqwest(request)).await.map_err(|e| {
            if e.is_timeout() {
                FetchError::Timeout
            } else {
                reqwest_error_to_fetch_error(e)
            }
        });

        info.track_connection();

        let mut resp = match result {
            Ok(response) => response,
            Err(e) => {
                return (Err(e), Some(info.finalize(0)));
            }
        };

        let status = resp.status();
        let headers = std::mem::take(resp.headers_mut());
        let extensions = std::mem::take(resp.extensions_mut());
        let version = resp.version();
        let result = resp.bytes().await.map_err(reqwest_error_to_fetch_error);

        info.track_response();

        let bytes = match result {
            Ok(bytes) => bytes,
            Err(e) => return (Err(e), Some(info.finalize(0))),
        };

        // reqwest transforms the body into a stream with Into
        let mut response = http::Response::new(OwnedOrSharedBytes::Shared(bytes));
        *response.status_mut() = status;
        *response.version_mut() = version;
        *response.extensions_mut() = extensions;
        *response.headers_mut() = headers;

        (Ok(response), Some(info.finalize(status.as_u16())))
    }

    async fn graphql_over_sse_stream(
        &self,
        request: FetchRequest<'_, Bytes>,
    ) -> FetchResult<impl Stream<Item = FetchResult<OwnedOrSharedBytes>> + Send + 'static> {
        let events = RequestBuilder::from_parts(self.client.clone(), into_reqwest(request))
            .eventsource()
            .unwrap()
            .map_err(|err| match err {
                reqwest_eventsource::Error::InvalidStatusCode(status_code, _) => {
                    FetchError::InvalidStatusCode(status_code)
                }
                err => FetchError::AnyError(err.to_string()),
            })
            .try_take_while(|event| {
                let is_complete = if let reqwest_eventsource::Event::Message(message) = event {
                    message.event == "complete"
                } else {
                    false
                };
                async move { Ok(!is_complete) }
            })
            .try_filter_map(|event| async move {
                let reqwest_eventsource::Event::Message(message) = event else {
                    return Ok(None);
                };
                if message.event == "next" {
                    Ok(Some(OwnedOrSharedBytes::Owned(message.data.into())))
                } else {
                    Err(FetchError::AnyError(format!("Unexpected event: {}", message.event)))
                }
            });
        Ok(events)
    }

    fn graphql_over_websocket_stream<T>(
        &self,
        request: FetchRequest<'_, T>,
    ) -> impl Future<Output = FetchResult<impl Stream<Item = FetchResult<serde_json::Value>> + Send + 'static>> + Send
    where
        T: serde::Serialize + Send,
    {
        use tungstenite::{client::IntoClientRequest, http::HeaderValue};

        // graphql_ws_client requires a 'static body which we can't provide.
        let body = serde_json::value::to_raw_value(&request.body).map_err(|err| FetchError::any(err.to_string()));
        let headers = Headers(request.headers);
        let mut ws_request = request.url.as_ref().into_client_request().unwrap();

        async move {
            let (connection, _) = {
                ws_request.headers_mut().insert(
                    "Sec-WebSocket-Protocol",
                    HeaderValue::from_str("graphql-transport-ws").unwrap(),
                );

                async_tungstenite::tokio::connect_async(ws_request)
                    .await
                    .map_err(FetchError::any)?
            };

            Ok(graphql_ws_client::Client::build(connection)
                .payload(headers)
                .map_err(FetchError::any)?
                .subscribe(WebsocketRequest(body?))
                .await
                .map_err(FetchError::any)?
                .map(|item| item.map_err(FetchError::any)))
        }
    }
}

fn reqwest_error_to_fetch_error(e: reqwest::Error) -> FetchError {
    FetchError::any(e.without_url())
}

fn into_reqwest(request: FetchRequest<'_, Bytes>) -> reqwest::Request {
    let mut req = reqwest::Request::new(request.method, request.url.into_owned());
    *req.headers_mut() = request.headers;
    *req.body_mut() = Some(request.body.into());
    *req.timeout_mut() = Some(request.timeout);
    req
}

struct Headers(http::HeaderMap);

impl serde::Serialize for Headers {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(None)?;
        for (name, value) in self.0.iter() {
            if let Ok(value) = value.to_str() {
                map.serialize_entry(name.as_str(), value)?;
            }
        }
        map.end()
    }
}

#[derive(serde::Serialize)]
struct WebsocketRequest<T>(T);

impl<T: serde::Serialize> graphql_ws_client::graphql::GraphqlOperation for WebsocketRequest<T> {
    type Response = serde_json::Value;

    type Error = FetchError;

    fn decode(&self, data: serde_json::Value) -> Result<Self::Response, Self::Error> {
        Ok(data)
    }
}
