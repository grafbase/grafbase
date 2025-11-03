pub(crate) mod cache;
pub mod mcp;
mod retry_budget;
mod runtime;

use ::runtime::{
    extension::{ContractsExtension as _, Token},
    operation_cache::OperationCache,
};
use bytes::Bytes;
use cache::CacheKey;
use error::{ErrorCode, ErrorResponse, GraphqlError};
use event_queue::EventQueue;
use futures::{StreamExt, TryFutureExt};
use futures_util::Stream;
use retry_budget::RetryBudgets;
use schema::Schema;
use std::{borrow::Cow, future::Future, sync::Arc};

use crate::{
    Body,
    execution::{EarlyHttpContext, Parts, RequestContext, StreamResponse},
    graphql_over_http::{ContentType, Http, ResponseFormat, StreamingResponseFormat},
    prepare::OperationDocument,
    response::Response,
    websocket::{self, InitPayload},
};
pub(crate) use runtime::*;

pub use runtime::Runtime;

pub struct ContractAwareEngine<R: Runtime> {
    // FIXME: do not expose this.
    pub no_contract: Arc<Engine<R>>,
    by_contract_key: quick_cache::sync::Cache<String, Arc<Engine<R>>>,
}

#[derive(Clone)]
pub struct RequestExtensions {
    pub token: Token,
    pub contract_key: Option<String>,
    pub event_queue: Arc<EventQueue>,
    pub hooks_context: Arc<[u8]>,
}

impl Default for RequestExtensions {
    fn default() -> Self {
        Self {
            token: Token::Anonymous,
            contract_key: None,
            event_queue: Arc::new(EventQueue::default()),
            hooks_context: Arc::new([]),
        }
    }
}

impl<R: Runtime> ContractAwareEngine<R> {
    pub fn new(schema: Arc<Schema>, runtime: R) -> Self {
        let no_contract = Arc::new(Engine::new(schema.clone(), runtime));
        let by_contract_key = quick_cache::sync::Cache::new(schema.config.contract_cache_max_size);
        Self {
            no_contract,
            by_contract_key,
        }
    }

    pub async fn execute<F>(&self, request: http::Request<F>) -> http::Response<Body>
    where
        F: Future<Output = Result<Bytes, (http::StatusCode, String)>> + Send,
    {
        let (parts, body) = match self.unpack_http_request(request) {
            Ok(unpacked) => unpacked,
            Err(response) => return response,
        };
        if let Some(key) = parts.extensions.contract_key.as_ref() {
            match self.get_engine_for_contract(key).await {
                Ok(engine) => engine.execute(parts, body).await,
                Err(err) => crate::http_error_response(
                    self.no_contract.schema.config.error_code_mapping.clone(),
                    parts.ctx.response_format,
                    err,
                ),
            }
        } else {
            self.no_contract.execute(parts, body).await
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
        let parts = Parts {
            ctx,
            headers: parts.headers,
            extensions: parts.extensions.remove().expect("Missing request extensions"),
        };

        if let Some(key) = parts.extensions.contract_key.as_ref() {
            let engine = self
                .get_engine_for_contract(key)
                .await
                .map_err(|err| err.into_message())?;
            engine.create_websocket_session(parts, payload).await
        } else {
            self.no_contract.create_websocket_session(parts, payload).await
        }
    }

    pub async fn get_schema(&self, parts: &http::request::Parts) -> Result<Arc<Schema>, Cow<'static, str>> {
        let extensions = parts
            .extensions
            .get::<RequestExtensions>()
            .expect("Missing request extensions");
        if let Some(key) = extensions.contract_key.as_ref() {
            let engine = self
                .get_engine_for_contract(key)
                .await
                .map_err(|err| err.into_message())?;
            Ok(engine.schema.clone())
        } else {
            Ok(self.no_contract.schema.clone())
        }
    }

    async fn get_engine_for_contract(&self, key: &str) -> Result<Arc<Engine<R>>, ErrorResponse> {
        match self.by_contract_key.get_value_or_guard_async(key).await {
            Ok(engine) => Ok(engine),
            Err(guard) => {
                let schema: Schema = self.no_contract.schema.as_ref().clone();
                let Some(mut schema) = self
                    .no_contract
                    .runtime
                    .extensions()
                    .construct(key.to_owned(), schema)
                    .await
                else {
                    return Err(ErrorResponse::internal_extension_error());
                };
                schema.rehash_with_contract_key(key);
                let schema = Arc::new(schema);
                let runtime = self
                    .no_contract
                    .runtime
                    .clone_and_adjust_for_contract(&schema)
                    .await
                    .map_err(|err| {
                        ErrorResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                            .with_error(GraphqlError::new(err, ErrorCode::ExtensionError))
                    })?;
                let engine = Arc::new(Engine::new(schema, runtime));
                let _ = guard.insert(engine.clone());
                Ok(engine)
            }
        }
    }
}

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
    pub(crate) fn new(schema: Arc<Schema>, runtime: R) -> Self {
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

        if count > 0 {
            tracing::info!("Finished warming {} operations", count);
        }
    }

    pub(crate) async fn execute<F>(
        self: &Arc<Self>,
        Parts {
            ctx,
            headers,
            extensions,
        }: Parts,
        body: F,
    ) -> http::Response<Body>
    where
        F: Future<Output = Result<Bytes, (http::StatusCode, String)>> + Send,
    {
        let context_fut = self
            .create_graphql_context(&ctx, headers, extensions, None)
            .map_err(|response| response);

        let request_fut = self
            .extract_well_formed_graphql_over_http_request(&ctx, body)
            .map_err(|response| response);

        // Retrieve the request body while processing the headers
        match self
            .with_gateway_timeout(async { futures::try_join!(context_fut, request_fut) })
            .await
            .unwrap_or_else(|| {
                Err(crate::execution::errors::response::gateway_timeout(
                    self.schema.config.error_code_mapping.clone(),
                ))
            }) {
            Ok((request_context, request)) => self.execute_well_formed_graphql_request(request_context, request).await,
            Err(response) => Http::error(ctx.response_format, response),
        }
    }

    pub(crate) async fn create_websocket_session(
        self: &Arc<Self>,
        Parts {
            ctx,
            headers,
            extensions,
        }: Parts,
        payload: InitPayload,
    ) -> Result<WebsocketSession<R>, Cow<'static, str>> {
        let request_context = self
            .create_graphql_context(&ctx, headers, extensions, Some(payload))
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
        self.runtime.with_timeout(self.schema.config.timeout, fut).await
    }
}

pub struct WebsocketSession<R: Runtime> {
    engine: Arc<Engine<R>>,
    request_context: Arc<RequestContext>,
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
