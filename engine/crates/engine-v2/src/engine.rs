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

/// The `Engine` struct serves as the main entry point for executing GraphQL operations.
///
/// It holds the necessary components required for processing GraphQL requests,
/// including the schema, runtime environment, authentication service, retry budgets,
/// operation cache, and the default response format.
///
/// # Type Parameters
///
/// * `R`: A type that implements the `Runtime` trait, providing specific runtime capabilities.
pub struct Engine<R: Runtime> {
    /// The GraphQL schema used for processing requests.
    ///
    /// The schema is wrapped in an `Arc` to enable shared ownership,
    /// allowing for self-contained responses while still providing
    /// access to the schema's underlying strings as needed.
    pub(crate) schema: Arc<Schema>,
    /// The runtime environment that allows execution of operations.
    pub(crate) runtime: R,
    /// The authentication service for managing authorization.
    auth: AuthService,
    /// The retry budgets for managing retries during request execution.
    retry_budgets: RetryBudgets,
    /// The cache for storing prepared GraphQL operations.
    operation_cache: <R::OperationCacheFactory as OperationCacheFactory>::Cache<Arc<PreparedOperation>>,
    /// The default format for responses.
    default_response_format: ResponseFormat,
}

impl<R: Runtime> Engine<R> {
    /// Creates a new instance of the `Engine` struct.
    ///
    /// This function initializes the `Engine` with the provided GraphQL schema and runtime
    /// environment. It also sets up the authentication service, retry budgets, and the operation cache.
    ///
    /// The `schema_version` is utilized as part of the operation cache key, ensuring that we only
    /// retrieve cached operations that correspond to the same schema version. If a schema version
    /// is not provided, a random one will be generated to maintain uniqueness.
    ///
    /// # Parameters
    ///
    /// * `schema`: An `Arc<Schema>` that represents the GraphQL schema used for processing requests.
    /// * `runtime`: An instance of the type implementing the `Runtime` trait, which provides
    ///   the necessary capabilities for operation execution.
    ///
    /// # Returns
    ///
    /// This function returns an instance of `Engine` with the given runtime.
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

    /// Executes a GraphQL operation based on the provided HTTP request.
    ///
    /// # Parameters
    ///
    /// * `request`: An HTTP request containing the GraphQL operation to be executed.
    ///
    /// # Returns
    ///
    /// This function returns an HTTP response containing the result of the executed
    /// GraphQL operation.
    ///
    /// # Errors
    ///
    /// Returns an HTTP response with an error status if the request could not be
    /// processed or if the operation execution fails.
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

    /// Creates a new WebSocket session for handling GraphQL subscriptions.
    ///
    /// This function initializes a WebSocket session and establishes the necessary
    /// request and hooks contexts. It processes the provided headers and sets the
    /// response format to be used in the session.
    ///
    /// # Parameters
    ///
    /// * `headers`: An HTTP header map containing any necessary authentication or
    ///   contextual information for the WebSocket session.
    ///
    /// # Returns
    ///
    /// Returns a `WebsocketSession` instance wrapped in a `Result`. On success,
    /// it provides the details for the established session. On failure, it
    /// returns an error message indicating what went wrong.
    ///
    /// # Errors
    ///
    /// The function may fail if the request context cannot be created, in which
    /// case it returns an error string indicating the problem.
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

/// The `WebsocketSession` struct represents a session for handling GraphQL subscriptions
/// over a WebSocket connection.
///
/// It holds references to the `Engine`, the request context, and the hooks context needed
/// for executing subscription requests. This struct is responsible for managing the state
/// of a subscription session and processing incoming events.
///
/// # Type Parameters
///
/// * `R`: A type that implements the `Runtime` trait, providing specific runtime capabilities.
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
    /// Executes a GraphQL operation in response to a WebSocket subscription event.
    ///
    /// This function processes the received `SubscribeEvent`, executing the associated
    /// GraphQL operation and returning a stream of WebSocket messages as the result. The
    /// messages can represent either an error or the next payload in the subscription.
    ///
    /// # Parameters
    ///
    /// * `event`: A `SubscribeEvent` that contains the subscription `id` and the
    ///   payload of the GraphQL operation to be executed.
    ///
    /// # Returns
    ///
    /// This function returns a stream of [`websocket::Message`] items. Each item represents
    /// either an error message or the next result from the execution of the subscription.
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
