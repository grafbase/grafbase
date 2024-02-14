//! graphql-ws-client glue code

use std::collections::BTreeMap;

use futures_util::Stream;
use http::Method;
use serde_json::json;

use super::{GqlRequest, GqlRequestBuilder};

impl GqlRequestBuilder<serde_json::Value> {
    pub async fn into_websocket_stream(
        mut self,
    ) -> Result<impl Stream<Item = serde_json::Value>, graphql_ws_client::Error> {
        use async_tungstenite::tungstenite::{client::IntoClientRequest, http::HeaderValue};
        use futures_util::StreamExt;

        let (mut request, payload_headers) = {
            let (client, request) = self.reqwest_builder.build_split();
            let request = request.unwrap();

            // make sure we can still use self below
            self.reqwest_builder = client.request(Method::GET, "http://example.com");

            let payload_headers = request
                .headers()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
                .collect::<BTreeMap<_, _>>();

            let mut url = request.url().clone();
            url.set_path("/ws");
            url.set_scheme("ws").unwrap();
            let request = url.into_client_request().unwrap();

            (request, payload_headers)
        };

        request.headers_mut().insert(
            "Sec-WebSocket-Protocol",
            HeaderValue::from_str("graphql-transport-ws").unwrap(),
        );

        let (connection, _) = async_tungstenite::tokio::connect_async(request).await.unwrap();

        Ok(graphql_ws_client::Client::build(connection)
            .payload(json!({ "headers": payload_headers }))?
            .subscribe(self.request)
            .await?
            .map(|item| item.unwrap()))
    }
}

impl graphql_ws_client::graphql::GraphqlOperation for GqlRequest {
    type Response = serde_json::Value;

    type Error = serde_json::Error;

    fn decode(&self, data: serde_json::Value) -> Result<Self::Response, Self::Error> {
        serde_json::from_value(data)
    }
}
