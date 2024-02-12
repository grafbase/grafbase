mod websockets;

use std::collections::HashMap;

use futures_util::stream::BoxStream;
use reqwest::header::HeaderValue;
use runtime::fetch::{FetchError, FetchRequest, FetchResponse, FetchResult, Fetcher, FetcherInner, GraphqlRequest};
use serde_json::json;

use self::websockets::StreamingRequest;

pub struct NativeFetcher {
    client: reqwest::Client,
}

impl NativeFetcher {
    pub fn runtime_fetcher() -> Fetcher {
        Fetcher::new(Self {
            client: reqwest::Client::new(),
        })
    }
}

#[async_trait::async_trait]
impl FetcherInner for NativeFetcher {
    async fn post(&self, request: FetchRequest<'_>) -> FetchResult<FetchResponse> {
        let response = self
            .client
            .post(request.url.clone())
            .body(request.json_body)
            .header("Content-Type", "application/json")
            .headers(
                request
                    .headers
                    .iter()
                    .filter_map(|(name, value)| Some((name.parse().ok()?, HeaderValue::from_str(value).ok()?)))
                    .collect(),
            )
            .send()
            .await
            .map_err(|e| FetchError::AnyError(e.to_string()))?;
        let bytes = response
            .bytes()
            .await
            .map_err(|e| FetchError::AnyError(e.to_string()))?;
        Ok(FetchResponse { bytes })
    }

    async fn stream(
        &self,
        request: GraphqlRequest<'_>,
    ) -> FetchResult<BoxStream<'static, Result<serde_json::Value, FetchError>>> {
        use async_tungstenite::tungstenite::{client::IntoClientRequest, http::HeaderValue};
        use futures_util::StreamExt;

        let (connection, _) = {
            let mut request = request.url.into_client_request().unwrap();
            request.headers_mut().insert(
                "Sec-WebSocket-Protocol",
                HeaderValue::from_str("graphql-transport-ws").unwrap(),
            );

            async_tungstenite::tokio::connect_async(request).await.unwrap()
        };

        let headers = request.headers.iter().copied().collect::<HashMap<_, _>>();

        Ok(graphql_ws_client::Client::build(connection)
            .payload(json!({"headers": headers}))
            .map_err(FetchError::any)?
            .subscribe(StreamingRequest::from(request))
            .await
            .map_err(FetchError::any)?
            .map(|item| item.map_err(FetchError::any))
            .boxed())
    }
}
