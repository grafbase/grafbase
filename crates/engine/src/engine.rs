use ::runtime::{hooks::Hooks as _, operation_cache::OperationCache};
use bytes::Bytes;
use cache::CacheKey;
use engine_auth::AuthService;
use futures::{StreamExt, TryFutureExt};
use futures_util::Stream;
use http::HeaderMap;
use retry_budget::RetryBudgets;
use schema::Schema;
use std::{borrow::Cow, future::Future, sync::Arc};

use crate::{
    graphql_over_http::{Http, ResponseFormat, StreamingResponseFormat},
    prepare::{CachedOperation, OperationDocument},
    response::Response,
    websocket::{self, InitPayload},
    Body, HooksExtension,
};
pub(crate) use execute::*;
pub(crate) use runtime::*;

pub(crate) mod cache;
mod errors;
mod execute;
mod retry_budget;
mod runtime;

pub use runtime::Runtime;

pub struct Engine<R: Runtime> {
    // We use an Arc for the schema to have a self-contained response which may still
    // needs access to the schema strings
    pub(crate) schema: Arc<Schema>,
    pub runtime: R,
    auth: AuthService,
    retry_budgets: RetryBudgets,
    default_response_format: ResponseFormat,
}

impl<R: Runtime> Engine<R> {
    /// schema_version is used in operation cache key which ensures we only retrieve cached
    /// operation for the same schema version. If none is provided, a random one is generated.
    pub async fn new(schema: Arc<Schema>, runtime: R) -> Self {
        let auth = AuthService::new_v2(
            schema.settings.auth_config.clone().unwrap_or_default(),
            runtime.kv().clone(),
        );

        Self {
            auth,
            retry_budgets: RetryBudgets::build(&schema),
            schema,
            runtime,
            // Could be coming from configuration one day
            default_response_format: ResponseFormat::application_json(),
        }
    }

    pub async fn warm<'doc, Doc>(self: &Arc<Self>, documents: impl IntoIterator<Item = Doc, IntoIter: Send> + Send)
    where
        Doc: Into<OperationDocument<'doc>> + Send,
    {
        tracing::info!("Warming operations");

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

        tracing::info!("Warming finished, {} operations were warmed", count);
    }

    pub async fn execute<F>(self: &Arc<Self>, request: http::Request<F>) -> http::Response<Body>
    where
        F: Future<Output = Result<Bytes, (http::StatusCode, String)>> + Send,
    {
        let (ctx, uri, headers, body) = match self.unpack_http_request(request) {
            Ok(req) => req,
            Err(response) => return response,
        };

        let request_context_fut = self
            .create_request_context(&ctx, headers, None)
            .map_err(|(response, context)| (response, Some(context)));

        let graphql_request_fut = self
            .extract_well_formed_graphql_over_http_request(&ctx, uri, body)
            .map_err(|response| (response, None));

        // Retrieve the request body while processing the headers
        match self
            .with_gateway_timeout(async { futures::try_join!(request_context_fut, graphql_request_fut) })
            .await
        {
            Some(Ok(((request_context, hooks_context), request))) => {
                self.execute_well_formed_graphql_request(request_context, hooks_context, request)
                    .await
            }
            Some(Err((mut response, hooks_context))) => {
                let context = hooks_context.unwrap_or_else(|| self.runtime.hooks().new_context());
                let on_operation_response_output = response.take_on_operation_response_output();
                let mut http_response = Http::error(ctx.response_format, response);

                http_response.extensions_mut().insert(HooksExtension::Single {
                    context,
                    on_operation_response_output,
                });

                http_response
            }
            None => Http::error(ctx.response_format, errors::response::gateway_timeout::<()>()),
        }
    }

    pub async fn create_websocket_session(
        self: &Arc<Self>,
        headers: HeaderMap,
        init_payload: InitPayload,
    ) -> Result<WebsocketSession<R>, Cow<'static, str>> {
        let response_format = ResponseFormat::Streaming(StreamingResponseFormat::GraphQLOverWebSocket);

        let ctx = EarlyHttpContext {
            method: http::Method::POST,
            response_format,
            include_grafbase_response_extension: false,
        };

        let (request_context, hooks_context) =
            match self.create_request_context(&ctx, headers, Some(init_payload)).await {
                Ok(context) => context,
                Err((response, _)) => {
                    return Err(response
                        .errors()
                        .first()
                        .map(|error| error.message.clone())
                        .unwrap_or("Internal server error".into()))
                }
            };

        Ok(WebsocketSession {
            engine: Arc::clone(self),
            request_context: Arc::new(request_context),
            hooks_context,
        })
    }
}

pub struct WebsocketSession<R: Runtime> {
    engine: Arc<Engine<R>>,
    request_context: Arc<RequestContext>,
    hooks_context: HooksContext<R>,
}

impl<R: Runtime> Clone for WebsocketSession<R> {
    fn clone(&self) -> Self {
        Self {
            engine: Arc::clone(&self.engine),
            request_context: Arc::clone(&self.request_context),
            hooks_context: self.hooks_context.clone(),
        }
    }
}

impl<R: Runtime> WebsocketSession<R> {
    pub fn execute(&self, event: websocket::SubscribeEvent) -> impl Stream<Item = websocket::Message<R>> {
        let websocket::SubscribeEvent { id, payload } = event;
        // TODO: Call a websocket hook?
        let StreamResponse { stream, .. } = self.engine.execute_websocket_well_formed_graphql_request(
            self.request_context.clone(),
            self.hooks_context.clone(),
            payload.0,
        );

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
