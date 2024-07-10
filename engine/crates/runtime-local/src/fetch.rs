mod websockets;

use std::collections::HashMap;

use futures_util::stream::BoxStream;
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
            .headers(request.headers.clone())
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
        use futures_util::StreamExt;
        use tungstenite::{client::IntoClientRequest, http::HeaderValue};

        let (connection, _) = {
            let mut request = request.url.into_client_request().unwrap();
            request.headers_mut().insert(
                "Sec-WebSocket-Protocol",
                HeaderValue::from_str("graphql-transport-ws").unwrap(),
            );

            async_tungstenite::tokio::connect_async(request)
                .await
                .map_err(FetchError::any)?
        };

        let headers: HashMap<_, _> = request
            .headers
            .iter()
            .flat_map(|(k, v)| v.to_str().map(|v| (k.as_str(), v)))
            .collect();

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
