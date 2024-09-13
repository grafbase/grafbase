use ::runtime::operation_cache::OperationCacheFactory;
use bytes::Bytes;
use futures::{StreamExt, TryFutureExt};
use futures_util::Stream;
use gateway_v2_auth::AuthService;
use retry_budget::RetryBudgets;
use schema::Schema;
use std::{borrow::Cow, future::Future, sync::Arc};

use crate::{
    graphql_over_http::{Http, ResponseFormat, StreamingResponseFormat},
    operation::PreparedOperation,
    response::Response,
    websocket, Body,
};
pub(crate) use execute::*;
pub(crate) use runtime::*;

mod cache;
mod error_responses;
mod execute;
mod retry_budget;
mod runtime;
mod trusted_documents;

pub use runtime::Runtime;

pub struct Engine<R: Runtime> {
    // We use an Arc for the schema to have a self-contained response which may still
    // needs access to the schema strings
    pub(crate) schema: Arc<Schema>,
    pub(crate) runtime: R,
    auth: AuthService,
    retry_budgets: RetryBudgets,
    operation_cache: <R::OperationCacheFactory as OperationCacheFactory>::Cache<Arc<PreparedOperation>>,
    default_response_format: ResponseFormat,
}

impl<R: Runtime> Engine<R> {
    /// schema_version is used in operation cache key which ensures we only retrieve cached
    /// operation for the same schema version. If none is provided, a random one is generated.
    pub async fn new(schema: Arc<Schema>, runtime: R) -> Self {
        let auth = gateway_v2_auth::AuthService::new_v2(
            schema.settings.auth_config.clone().unwrap_or_default(),
            runtime.kv().clone(),
        );

        Self {
            auth,
            retry_budgets: RetryBudgets::build(&schema),
            operation_cache: runtime.operation_cache_factory().create().await,
            schema,
            runtime,
            // Could be coming from configuration one day
            default_response_format: ResponseFormat::application_json(),
        }
    }

    pub async fn execute<F>(self: &Arc<Self>, request: http::Request<F>) -> http::Response<Body>
    where
        F: Future<Output = Result<Bytes, (http::StatusCode, String)>> + Send,
    {
        let (
            http::request::Parts {
                method, headers, uri, ..
            },
            body,
            response_format,
        ) = match self.unpack_http_request(request) {
            Ok(req) => req,
            Err(response) => return response,
        };

        let request_context_fut = self
            .create_request_context(!method.is_safe(), headers, response_format)
            .map_err(|response| Http::error(response_format, response));

        let graphql_request_fut =
            self.extract_well_formed_graphql_over_http_request(method, uri, response_format, body);

        // Retrieve the request body while processing the headers
        match futures::try_join!(request_context_fut, graphql_request_fut) {
            Ok(((request_context, hooks_context), request)) => {
                self.execute_well_formed_graphql_request(request_context, hooks_context, request)
                    .await
            }
            Err(response) => response,
        }
    }

    pub async fn create_websocket_session(
        self: &Arc<Self>,
        headers: http::HeaderMap,
    ) -> Result<WebsocketSession<R>, Cow<'static, str>> {
        let response_format = ResponseFormat::Streaming(StreamingResponseFormat::GraphQLOverWebSocket);

        let (request_context, hooks_context) = match self.create_request_context(true, headers, response_format).await {
            Ok(context) => context,
            Err(response) => {
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
    pub fn execute(&self, event: websocket::SubscribeEvent) -> impl Stream<Item = websocket::Message> {
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
