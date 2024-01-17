//! graphql-ws-client glue code

use futures_util::Stream;
use http::Method;
use serde_json::json;

use super::GqlRequestBuilder;

impl<Response> GqlRequestBuilder<Response>
where
    Response: serde::de::DeserializeOwned + 'static,
{
    pub async fn into_websocket_stream(mut self) -> impl Stream<Item = Response> {
        use async_tungstenite::tungstenite::{client::IntoClientRequest, http::HeaderValue};
        use futures_util::StreamExt;

        let mut client = {
            let mut request = {
                let (client, request) = self.reqwest_builder.build_split();

                // make sure we can still use self below
                self.reqwest_builder = client.request(Method::GET, "http://example.com");

                let mut url = request.unwrap().url().clone();
                url.set_path("/ws");
                url.set_scheme("ws").unwrap();
                url.into_client_request().unwrap()
            };

            request.headers_mut().insert(
                "Sec-WebSocket-Protocol",
                HeaderValue::from_str("graphql-transport-ws").unwrap(),
            );

            let (connection, _) = async_tungstenite::tokio::connect_async(request).await.unwrap();

            let (sink, stream) = connection.split();

            graphql_ws_client::AsyncWebsocketClientBuilder::<CliGraphqlClient>::new()
                .build(stream, sink, TokioSpawner::current())
                .await
                .unwrap()
        };

        client.streaming_operation(self).await.unwrap().map(move |item| {
            // Ignore this next line, I'm just tricking rust into
            // moving the client into this closure.
            let _client = &client;

            item.unwrap()
        })
    }
}

pub struct CliGraphqlClient;

impl graphql_ws_client::graphql::GraphqlClient for CliGraphqlClient {
    type Response = serde_json::Value;

    type DecodeError = serde_json::Error;

    fn error_response(errors: Vec<serde_json::Value>) -> Result<Self::Response, Self::DecodeError> {
        Ok(json!({"errors": errors}))
    }
}

impl<T> graphql_ws_client::graphql::GraphqlOperation for GqlRequestBuilder<T>
where
    T: serde::de::DeserializeOwned,
{
    type GenericResponse = serde_json::Value;

    type Response = T;

    type Error = serde_json::Error;

    fn decode(&self, data: Self::GenericResponse) -> Result<Self::Response, Self::Error> {
        serde_json::from_value(data)
    }
}

pub struct TokioSpawner(tokio::runtime::Handle);

impl TokioSpawner {
    pub fn new(handle: tokio::runtime::Handle) -> Self {
        TokioSpawner(handle)
    }

    pub fn current() -> Self {
        TokioSpawner::new(tokio::runtime::Handle::current())
    }
}

impl futures_util::task::Spawn for TokioSpawner {
    fn spawn_obj(&self, obj: futures_util::task::FutureObj<'static, ()>) -> Result<(), futures_util::task::SpawnError> {
        self.0.spawn(obj);
        Ok(())
    }
}
