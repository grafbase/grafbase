//! graphql-ws-client glue code

use std::future::IntoFuture;

use futures::{future::BoxFuture, stream::BoxStream};
use url::Url;

use super::{GraphQlRequest, GraphqlResponse};

pub struct WebsocketRequest {
    pub(super) gql: GraphQlRequest,
    pub(super) headers: http::HeaderMap,
    pub(super) init_payload: Option<serde_json::Value>,
    pub(super) router: axum::Router<()>,
    pub(super) path: &'static str,
}

impl WebsocketRequest {
    pub fn by_client(self, name: &'static str, version: &'static str) -> Self {
        self.header("x-grafbase-client-name", name)
            .header("x-grafbase-client-version", version)
    }

    pub fn header<Name, Value>(mut self, name: Name, value: Value) -> Self
    where
        Name: TryInto<http::HeaderName, Error: std::fmt::Debug>,
        Value: TryInto<http::HeaderValue, Error: std::fmt::Debug>,
    {
        self.headers.insert(name.try_into().unwrap(), value.try_into().unwrap());
        self
    }

    pub fn header_append<Name, Value>(mut self, name: Name, value: Value) -> Self
    where
        Name: TryInto<http::HeaderName, Error: std::fmt::Debug>,
        Value: TryInto<http::HeaderValue, Error: std::fmt::Debug>,
    {
        self.headers.append(name.try_into().unwrap(), value.try_into().unwrap());
        self
    }

    pub fn variables(mut self, variables: impl serde::Serialize) -> Self {
        self.gql.variables = Some(serde_json::to_value(variables).expect("variables to be serializable"));
        self
    }

    pub fn extensions(mut self, extensions: impl serde::Serialize) -> Self {
        self.gql.extensions =
            serde_json::from_value(serde_json::to_value(extensions).expect("extensions to be serializable"))
                .expect("extensions to be deserializable");
        self
    }

    pub fn operation_name(mut self, name: impl Into<String>) -> Self {
        self.gql.operation_name = Some(name.into());
        self
    }

    pub fn init_payload(mut self, payload: serde_json::Value) -> Self {
        self.init_payload = Some(payload);
        self
    }

    pub fn with_path(mut self, path: &'static str) -> Self {
        self.path = path;
        self
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WebsocketRequestError {
    #[error(transparent)]
    WsClient(#[from] graphql_ws_client::Error),
    #[error(transparent)]
    Tungstenite(#[from] async_tungstenite::tungstenite::Error),
}

impl IntoFuture for WebsocketRequest {
    type Output = Result<BoxStream<'static, GraphqlResponse>, WebsocketRequestError>;
    type IntoFuture = BoxFuture<'static, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        use async_tungstenite::tungstenite::{client::IntoClientRequest, http::HeaderValue};
        use futures_util::StreamExt;

        Box::pin(async move {
            let handler = self.router.into_make_service();
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();

            let mut url: Url = format!("http://{}", listener.local_addr().unwrap()).parse().unwrap();

            url.set_path(self.path);
            url.set_scheme("ws").unwrap();

            // It's fine to leave this running since nextest is process-per-test.
            tokio::spawn(axum::serve(listener, handler).into_future());

            let mut request = url.as_ref().into_client_request().unwrap();

            request.headers_mut().extend(self.headers);
            request.headers_mut().insert(
                http::header::SEC_WEBSOCKET_PROTOCOL,
                HeaderValue::from_str("graphql-transport-ws").unwrap(),
            );

            let (connection, _) = async_tungstenite::tokio::connect_async(request).await?;

            let (client, actor) = graphql_ws_client::Client::build(connection)
                .payload(self.init_payload.unwrap_or_default())?
                .await?;

            tokio::spawn(actor.into_future());

            let stream: BoxStream<'_, _> = Box::pin(
                client
                    .subscribe(self.gql)
                    .await?
                    .map(move |item| -> GraphqlResponse { item.unwrap() }),
            );

            Ok::<_, WebsocketRequestError>(stream)
        })
    }
}

impl graphql_ws_client::graphql::GraphqlOperation for GraphQlRequest {
    type Response = GraphqlResponse;
    type Error = serde_json::Error;

    fn decode(&self, data: serde_json::Value) -> Result<Self::Response, Self::Error> {
        Ok(GraphqlResponse {
            status: Default::default(),
            headers: Default::default(),
            body: data,
        })
    }
}
