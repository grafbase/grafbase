pub(crate) mod cache;
pub mod mcp;
mod retry_budget;
mod runtime;

use ::runtime::operation_cache::OperationCache;
use bytes::Bytes;
use cache::CacheKey;
use futures::{StreamExt, TryFutureExt};
use futures_util::Stream;
use retry_budget::RetryBudgets;
use schema::Schema;
use std::{borrow::Cow, future::Future, sync::Arc};

use crate::{
    Body,
    execution::{EarlyHttpContext, RequestContext, StreamResponse},
    graphql_over_http::{ContentType, Http, ResponseFormat, StreamingResponseFormat},
    prepare::OperationDocument,
    response::Response,
    websocket::{self, InitPayload},
};
pub(crate) use runtime::*;

pub use runtime::Runtime;

pub struct Engine<R: Runtime> {
    // We use an Arc for the schema to have a self-contained response which may still
    // needs access to the schema strings
    pub schema: Arc<Schema>,
    pub runtime: R,
    pub(crate) retry_budgets: RetryBudgets,
}

impl<R: Runtime> Engine<R> {
    /// schema_version is used in operation cache key which ensures we only retrieve cached
    /// operation for the same schema version. If none is provided, a random one is generated.
    pub async fn new(schema: Arc<Schema>, runtime: R) -> Self {
        Self {
            retry_budgets: RetryBudgets::build(&schema),
            schema,
            runtime,
        }
    }

    pub async fn warm<'doc, Doc>(self: &Arc<Self>, documents: impl IntoIterator<Item = Doc, IntoIter: Send> + Send)
    where
        Doc: Into<OperationDocument<'doc>> + Send,
    {
        tracing::debug!("Warming operations");

        let mut count = 0;

        for document in documents {
            let document: OperationDocument<'_> = document.into();
            let name = document.operation_name().map(|s| s.to_owned());
            let cache_key = CacheKey::document(&self.schema, &document.key).to_string();

            match self.warm_operation(document) {
                Ok(cached) => {
                    count += 1;
                    self.runtime.operation_cache().insert(cache_key, Arc::new(cached)).await;
                }
                Err(err) => {
                    // Ensure we're yield regularly.
                    futures_lite::future::yield_now().await;
                    tracing::warn!("Could not plan operation {}: {err}", name.unwrap_or_default());
                }
            }
        }

        tracing::info!("Finished warming {} operations", count);
    }

    pub async fn execute<F>(self: &Arc<Self>, mut request: http::Request<F>) -> http::Response<Body>
    where
        F: Future<Output = Result<Bytes, (http::StatusCode, String)>> + Send,
    {
        // Hey, you. If this returns the default, go check in the engine-axum crate the hooks middleware.
        // Did you insert the context there?
        let extension_ctx = request
            .extensions_mut()
            .remove::<ExtensionContext<R>>()
            // FIXME: mcp should should go through the hooks...
            .unwrap_or_default();

        let (ctx, headers, body) = match self.unpack_http_request(request) {
            Ok(req) => req,
            Err(response) => return response,
        };

        let context_fut = self
            .create_graphql_context(&ctx, headers, None, extension_ctx)
            .map_err(|response| response);

        let request_fut = self
            .extract_well_formed_graphql_over_http_request(&ctx, body)
            .map_err(|response| response);

        // Retrieve the request body while processing the headers
        match self
            .with_gateway_timeout(async { futures::try_join!(context_fut, request_fut) })
            .await
            .unwrap_or_else(|| Err(crate::execution::errors::response::gateway_timeout()))
        {
            Ok((request_context, request)) => self.execute_well_formed_graphql_request(request_context, request).await,
            Err(response) => Http::error(ctx.response_format, response),
        }
    }

    pub async fn create_websocket_session(
        self: &Arc<Self>,
        mut parts: http::request::Parts,
        payload: InitPayload,
    ) -> Result<WebsocketSession<R>, Cow<'static, str>> {
        let ctx = EarlyHttpContext {
            method: parts.method,
            uri: parts.uri,
            can_mutate: true,
            response_format: ResponseFormat::Streaming(StreamingResponseFormat::GraphQLOverWebSocket),
            include_grafbase_response_extension: false,
            include_mcp_response_extension: false,
            content_type: ContentType::Json,
        };

        // Hey, you. If this returns the default, go check in the engine-axum crate the hooks middleware.
        // Did you insert the context there?
        let extension_context = parts
            .extensions
            .remove::<ExtensionContext<R>>()
            .expect("Missing Wasm context");

        let request_context = self
            .create_graphql_context(&ctx, parts.headers, Some(payload), extension_context)
            .await
            .map_err(|response| {
                response
                    .pre_execution_errors()
                    .first()
                    .map(|error| error.message.clone())
                    .unwrap_or("Internal server error".into())
            })?;

        Ok(WebsocketSession {
            engine: self.clone(),
            request_context,
        })
    }

    pub(crate) async fn with_gateway_timeout<T>(&self, fut: impl Future<Output = T> + Send) -> Option<T> {
        self.runtime.with_timeout(self.schema.settings.timeout, fut).await
    }
}

pub struct WebsocketSession<R: Runtime> {
    engine: Arc<Engine<R>>,
    request_context: Arc<RequestContext<ExtensionContext<R>>>,
}

impl<R: Runtime> Clone for WebsocketSession<R> {
    fn clone(&self) -> Self {
        Self {
            engine: self.engine.clone(),
            request_context: self.request_context.clone(),
        }
    }
}

impl<R: Runtime> WebsocketSession<R> {
    pub fn execute(&self, event: websocket::SubscribeEvent) -> impl Stream<Item = websocket::Message> + 'static {
        let websocket::SubscribeEvent { id, payload } = event;
        // TODO: Call a websocket hook?
        let StreamResponse { stream, .. } = self
            .engine
            .execute_websocket_well_formed_graphql_request(self.request_context.clone(), payload.0);

        stream.map(move |response| match response {
            Response::RefusedRequest(_) => websocket::Message::Error {
                id: id.clone(),
                payload: websocket::ResponsePayload(response),
            },
            response => websocket::Message::Next {
                id: id.clone(),
                payload: websocket::ResponsePayload(response),
            },
        })
    }
}
