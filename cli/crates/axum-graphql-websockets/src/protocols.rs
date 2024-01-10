use std::str::FromStr;

use axum::{
    extract::FromRequestParts,
    http::{self, request::Parts, StatusCode},
};

const GRAPHQL_WS_ID: &str = "graphql-transport-ws";
pub const SUPPORTED_PROTOCOL_IDS: &[&str] = &[GRAPHQL_WS_ID];

/// A GraphQL protocol extractor.
///
/// It extract GraphQL protocol from `SEC_WEBSOCKET_PROTOCOL` header.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WebsocketProtocols {
    GraphQlWs,
}

impl FromStr for WebsocketProtocols {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            GRAPHQL_WS_ID => Ok(WebsocketProtocols::GraphQlWs),
            _ => Err(()),
        }
    }
}

#[async_trait::async_trait]
impl<S> FromRequestParts<S> for WebsocketProtocols
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
                    .find_map(|p| WebsocketProtocols::from_str(p.trim()).ok())
            })
            .map(Self)
            .ok_or(StatusCode::BAD_REQUEST)
    }
}
