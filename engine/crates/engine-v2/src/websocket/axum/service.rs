//! Axum specific websocket integration code

use std::{
    convert::Infallible,
    str::FromStr,
    task::{Context, Poll},
};

use axum::{
    body::{Body, HttpBody},
    extract::{ws, FromRequestParts, WebSocketUpgrade},
    http::{self, request::Parts, Request, Response, StatusCode},
    response::IntoResponse,
};
use futures_util::future::BoxFuture;
use tower_service::Service;

use super::WebsocketSender;

/// A tower service that accepts websocket connections, passing them to the provided sender
#[derive(Clone)]
pub struct WebsocketService {
    sender: WebsocketSender,
}

impl WebsocketService {
    pub fn new(sender: WebsocketSender) -> Self {
        Self { sender }
    }
}

impl<B> Service<Request<B>> for WebsocketService
where
    B: HttpBody + Send + 'static,
{
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let sender = self.sender.clone();

        Box::pin(async move {
            let (mut parts, _body) = req.into_parts();

            match WebsocketProtocol::from_request_parts(&mut parts, &()).await {
                Ok(_) => {}
                Err(err) => return Ok(err.into_response()),
            };
            let upgrade = match WebSocketUpgrade::from_request_parts(&mut parts, &()).await {
                Ok(protocol) => protocol,
                Err(err) => return Ok(err.into_response()),
            };

            let resp = upgrade
                .protocols(SUPPORTED_PROTOCOL_IDS)
                .on_upgrade(move |websocket| async move {
                    sender.send(websocket).await.ok();
                });

            Ok(resp.into_response())
        })
    }
}

const GRAPHQL_WS_ID: &str = "graphql-transport-ws";
const SUPPORTED_PROTOCOL_IDS: [&str; 1] = [GRAPHQL_WS_ID];

/// A GraphQL protocol extractor.
///
/// It extract GraphQL protocol from `SEC_WEBSOCKET_PROTOCOL` header.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WebsocketProtocol {
    GraphQlWs,
}

impl FromStr for WebsocketProtocol {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            GRAPHQL_WS_ID => Ok(WebsocketProtocol::GraphQlWs),
            _ => Err(()),
        }
    }
}

#[async_trait::async_trait]
impl<S> FromRequestParts<S> for WebsocketProtocol
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .headers
            .get(http::header::SEC_WEBSOCKET_PROTOCOL)
            .and_then(|value| value.to_str().ok())
            .and_then(|protocols| {
                protocols
                    .split(',')
                    .find_map(|p| WebsocketProtocol::from_str(p.trim()).ok())
            })
            .ok_or(StatusCode::BAD_REQUEST)
    }
}

pub trait MessageConvert {
    fn to_axum_message(self) -> Result<ws::Message, serde_json::Error>;
}

impl MessageConvert for crate::websocket::messages::Message {
    fn to_axum_message(self) -> Result<ws::Message, serde_json::Error> {
        match self {
            crate::websocket::messages::Message::Close { code, reason } => {
                Ok(ws::Message::Close(Some(ws::CloseFrame {
                    code,
                    reason: reason.into(),
                })))
            }
            message => Ok(ws::Message::Text(serde_json::to_string(&message)?)),
        }
    }
}
