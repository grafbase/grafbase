//! A mock GraphQL server for testing the GraphQL connector

use std::{sync::Arc, time::Duration};

use async_graphql_axum::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use axum::{extract::State, http::HeaderMap, routing::post, Router};
use tokio::sync::mpsc;

mod almost_empty;
mod disingenuous;
mod echo;
mod error_schema;
mod fake_github;
mod federation;
mod state_mutation;

pub use {
    almost_empty::AlmostEmptySchema,
    disingenuous::DisingenuousSchema,
    echo::EchoSchema,
    error_schema::ErrorSchema,
    fake_github::FakeGithubSchema,
    federation::{FakeFederationAccountsSchema, FakeFederationProductsSchema, FakeFederationReviewsSchema},
    state_mutation::StateMutationSchema,
};

pub struct MockGraphQlServer {
    pub schema: Arc<dyn Schema>,
    received_requests: mpsc::UnboundedReceiver<async_graphql::Request>,
    shutdown: Option<tokio::sync::oneshot::Sender<()>>,
    port: u16,
}

impl Drop for MockGraphQlServer {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            shutdown.send(()).ok();
        }
    }
}

impl MockGraphQlServer {
    pub async fn new(schema: impl Schema + 'static) -> MockGraphQlServer {
        Self::new_impl(Arc::new(schema)).await
    }

    async fn new_impl(schema: Arc<dyn Schema>) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        let state = AppState {
            schema: schema.clone(),
            requests: sender,
        };
        let app = Router::new()
            .route("/", post(graphql_handler))
            .route_service("/ws", GraphQLSubscription::new(SchemaExecutor(schema.clone())))
            .with_state(state);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let (shutdown_sender, shutdown_receiver) = tokio::sync::oneshot::channel::<()>();

        tokio::spawn(async move {
            axum::serve(listener, app.with_state(()))
                .with_graceful_shutdown(async move {
                    shutdown_receiver.await.ok();
                })
                .await
                .unwrap();
        });

        // Give the server time to start
        tokio::time::sleep(Duration::from_millis(20)).await;

        MockGraphQlServer {
            schema,
            shutdown: Some(shutdown_sender),
            received_requests: receiver,
            port,
        }
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn url(&self) -> String {
        format!("http://localhost:{}", self.port)
    }

    pub fn websocket_url(&self) -> String {
        format!("ws://localhost:{}/ws", self.port)
    }

    pub async fn drain_requests(&mut self) -> impl Iterator<Item = async_graphql::Request> + '_ {
        std::iter::from_fn(|| self.received_requests.try_recv().ok())
    }
}

async fn graphql_handler(State(state): State<AppState>, headers: HeaderMap, req: GraphQLRequest) -> GraphQLResponse {
    let req = req.into_inner();

    // Record the request incase tests want to inspect it.
    // async_graphql::Request isn't clone so we do a deser roundtrip instead
    state
        .requests
        .send(serde_json::from_value(serde_json::to_value(&req).unwrap()).unwrap())
        .ok();

    let headers = headers
        .into_iter()
        .map(|(name, value)| {
            (
                name.map(|name| name.to_string()).unwrap_or_default(),
                String::from_utf8_lossy(value.as_bytes()).to_string(),
            )
        })
        .collect();

    let response: GraphQLResponse = state.schema.execute(headers, req).await.into();
    response
}

#[derive(Clone)]
struct AppState {
    schema: Arc<dyn Schema>,
    requests: mpsc::UnboundedSender<async_graphql::Request>,
}

/// Creating a trait for schema so we can use it as a trait object and avoid
/// making everything generic over Query, Mutation & Subscription params
#[async_trait::async_trait]
pub trait Schema: Send + Sync {
    async fn execute(&self, headers: Vec<(String, String)>, request: async_graphql::Request)
        -> async_graphql::Response;

    fn execute_stream(
        &self,
        request: async_graphql::Request,
    ) -> futures::stream::BoxStream<'static, async_graphql::Response>;

    fn sdl(&self) -> String;
}

#[async_trait::async_trait]
impl<Q, M, S> Schema for async_graphql::Schema<Q, M, S>
where
    Q: async_graphql::ObjectType + 'static,
    M: async_graphql::ObjectType + 'static,
    S: async_graphql::SubscriptionType + 'static,
{
    async fn execute(
        &self,
        _headers: Vec<(String, String)>,
        request: async_graphql::Request,
    ) -> async_graphql::Response {
        async_graphql::Schema::execute(self, request).await
    }

    fn execute_stream(
        &self,
        request: async_graphql::Request,
    ) -> futures::stream::BoxStream<'static, async_graphql::Response> {
        Box::pin(async_graphql::Schema::execute_stream(self, request))
    }

    fn sdl(&self) -> String {
        self.sdl_with_options(async_graphql::SDLExportOptions::new())
    }
}

#[derive(Clone)]
pub struct SchemaExecutor(Arc<dyn Schema>);

impl async_graphql::Executor for SchemaExecutor {
    /// Execute a GraphQL query.
    fn execute(
        &self,
        request: async_graphql::Request,
    ) -> impl futures::future::Future<Output = async_graphql::Response> {
        self.0.execute(Default::default(), request)
    }

    /// Execute a GraphQL subscription with session data.
    fn execute_stream(
        &self,
        request: async_graphql::Request,
        _session_data: Option<Arc<async_graphql::Data>>,
    ) -> futures::stream::BoxStream<'static, async_graphql::Response> {
        self.0.execute_stream(request)
    }
}
