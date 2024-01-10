use std::{
    borrow::Cow,
    convert::Infallible,
    future::Future,
    str::FromStr,
    task::{Context, Poll},
};

use axum::{
    body::{Body, HttpBody},
    extract::{
        ws::{CloseFrame, Message},
        FromRequestParts, WebSocketUpgrade,
    },
    http::{self, request::Parts, Request, Response, StatusCode},
    response::IntoResponse,
    Error,
};
use executor::Executor;
use futures_util::{
    future,
    future::{BoxFuture, Ready},
    stream::{SplitSink, SplitStream},
    Sink, SinkExt, Stream, StreamExt,
};
use tower_service::Service;

use crate::protocols::{WebsocketProtocols, SUPPORTED_PROTOCOL_IDS};

/// A tower service that provides GraphQL subscription support to axum
#[derive(Clone)]
pub struct SubscriptionService<E> {
    executor: E,
}

impl<E> SubscriptionService<E>
where
    E: Executor,
{
    /// Create a GraphQL subscription service.
    pub fn new(executor: E) -> Self {
        Self { executor }
    }
}

impl<B, E> Service<Request<B>> for SubscriptionService<E>
where
    B: HttpBody + Send + 'static,
    E: Executor,
{
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let executor = self.executor.clone();

        Box::pin(async move {
            let (mut parts, _body) = req.into_parts();

            let protocol = match WebsocketProtocols::from_request_parts(&mut parts, &()).await {
                Ok(protocol) => protocol,
                Err(err) => return Ok(err.into_response()),
            };
            let upgrade = match WebSocketUpgrade::from_request_parts(&mut parts, &()).await {
                Ok(protocol) => protocol,
                Err(err) => return Ok(err.into_response()),
            };

            let executor = executor.clone();

            let resp = upgrade
                .protocols(SUPPORTED_PROTOCOL_IDS)
                .on_upgrade(move |stream| GraphQLWebSocket::new(stream, executor, protocol).serve());
            Ok(resp.into_response())
        })
    }
}
