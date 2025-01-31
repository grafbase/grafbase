//! Adapted from [this async-graphql-axum module](https://github.com/async-graphql/async-graphql/blob/33282a1bb54912aff2c377fa216df758021fda2c/integrations/axum/src/subscription.rs), but injecting our own RequestContext.

use std::{
    convert::Infallible,
    task::{Context, Poll},
};

use async_graphql::{http::ALL_WEBSOCKET_PROTOCOLS, Data};
use async_graphql_axum::{GraphQLProtocol, GraphQLWebSocket};
use axum::{
    body::{Body, HttpBody},
    extract::{FromRequestParts as _, Request, WebSocketUpgrade},
    response::{IntoResponse as _, Response},
};
use futures_util::future::BoxFuture;
use tower::Service;

use crate::SchemaExecutor;

#[derive(Clone)]
pub(crate) struct SubscriptionService {
    schema: SchemaExecutor,
}

impl SubscriptionService {
    pub(crate) fn new(schema: SchemaExecutor) -> Self {
        Self { schema }
    }
}

impl<B> Service<Request<B>> for SubscriptionService
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
        let schema = self.schema.clone();

        Box::pin(async move {
            let (mut parts, _body) = req.into_parts();

            let protocol = match GraphQLProtocol::from_request_parts(&mut parts, &()).await {
                Ok(protocol) => protocol,
                Err(err) => return Ok(err.into_response()),
            };
            let upgrade = match WebSocketUpgrade::from_request_parts(&mut parts, &()).await {
                Ok(protocol) => protocol,
                Err(err) => return Ok(err.into_response()),
            };

            let resp = upgrade.protocols(ALL_WEBSOCKET_PROTOCOLS).on_upgrade(move |stream| {
                GraphQLWebSocket::new(stream, schema, protocol)
                    .on_connection_init(move |payload| async move {
                        let mut out = Data::default();
                        out.insert(ConnectionInitPayload(payload));
                        out.insert(parts.headers);
                        Ok(out)
                    })
                    .serve()
            });

            Ok(resp.into_response())
        })
    }
}

#[derive(Debug)]
pub(crate) struct ConnectionInitPayload(pub(crate) serde_json::Value);
