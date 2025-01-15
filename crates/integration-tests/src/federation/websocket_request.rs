//! graphql-ws-client glue code

use std::future::IntoFuture;

use futures::{future::BoxFuture, stream::BoxStream};
use url::Url;

use super::{GraphQlRequest, GraphqlResponse};

pub struct WebsocketRequest {
    pub(super) request: GraphQlRequest,
    pub(super) init_payload: Option<serde_json::Value>,
    pub(super) router: axum::Router<()>,
}

impl IntoFuture for WebsocketRequest {
    type Output = Result<BoxStream<'static, GraphqlResponse>, graphql_ws_client::Error>;
    type IntoFuture = BoxFuture<'static, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        use async_tungstenite::tungstenite::{client::IntoClientRequest, http::HeaderValue};
        use futures_util::StreamExt;

        Box::pin(async move {
            let handler = self.router.into_make_service();
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();

            let mut url: Url = format!("http://{}", listener.local_addr().unwrap()).parse().unwrap();
            url.set_path("/ws");
            url.set_scheme("ws").unwrap();

            // It's fine to leave this running since nextest is process-per-test.
            tokio::spawn(axum::serve(listener, handler).into_future());

            let mut request = url.as_ref().into_client_request().unwrap();

            request.headers_mut().insert(
                "Sec-WebSocket-Protocol",
                HeaderValue::from_str("graphql-transport-ws").unwrap(),
            );

            let (connection, _) = async_tungstenite::tokio::connect_async(request).await.unwrap();

            let (client, actor) = graphql_ws_client::Client::build(connection)
                .payload(self.init_payload.unwrap_or_else(|| serde_json::Value::Null))?
                .await?;

            tokio::spawn(actor.into_future());

            let stream: BoxStream<'_, _> = Box::pin(
                client
                    .subscribe(self.request)
                    .await?
                    .map(move |item| -> GraphqlResponse { item.unwrap() }),
            );

            Ok::<_, graphql_ws_client::Error>(stream)
        })
    }
}

impl graphql_ws_client::graphql::GraphqlOperation for GraphQlRequest {
    type Response = GraphqlResponse;
    type Error = serde_json::Error;

    fn decode(&self, data: serde_json::Value) -> Result<Self::Response, Self::Error> {
        serde_json::from_value(data)
    }
}
