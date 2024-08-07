//! A mock GraphQL server for testing the GraphQL connector

use std::{sync::Arc, time::Duration};

use async_graphql_axum::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use axum::{extract::State, http::HeaderMap, response::IntoResponse, routing::post, Router};
use futures::Future;
use serde::ser::SerializeMap;
use url::Url;

mod almost_empty;
mod echo;
mod error_schema;
mod fake_github;
mod federation;
mod secure;
mod slow;
mod state_mutation;
mod tea_shop;

pub use {
    almost_empty::AlmostEmptySchema, echo::EchoSchema, error_schema::ErrorSchema, fake_github::FakeGithubSchema,
    federation::*, secure::SecureSchema, slow::SlowSchema, state_mutation::StateMutationSchema, tea_shop::TeaShop,
};

#[derive(Debug)]
pub struct ReceivedRequest {
    pub headers: http::HeaderMap,
    pub body: async_graphql::Request,
}

impl serde::Serialize for ReceivedRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;
        let mut headers = self
            .headers
            .iter()
            .map(|(name, value)| (name.to_string(), String::from_utf8_lossy(value.as_bytes()).into_owned()))
            .collect::<Vec<_>>();
        headers.sort_unstable();
        map.serialize_entry("headers", &headers)?;
        map.serialize_entry("body", &self.body)?;
        map.end()
    }
}

impl std::ops::Deref for ReceivedRequest {
    type Target = async_graphql::Request;
    fn deref(&self) -> &Self::Target {
        &self.body
    }
}

pub struct MockGraphQlServer {
    state: AppState,
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
        let state = AppState {
            schema: schema.clone(),
            received_requests: Default::default(),
            next_responses: Default::default(),
        };

        let app = Router::new()
            .route("/", post(graphql_handler))
            .route_service("/ws", GraphQLSubscription::new(SchemaExecutor(schema)))
            .with_state(state.clone());

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
            state,
            shutdown: Some(shutdown_sender),
            port,
        }
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn url(&self) -> Url {
        format!("http://127.0.0.1:{}", self.port).parse().unwrap()
    }

    pub fn sdl(&self) -> String {
        self.state.schema.sdl()
    }

    pub fn websocket_url(&self) -> Url {
        format!("ws://127.0.0.1:{}/ws", self.port).parse().unwrap()
    }

    pub fn drain_received_requests(&self) -> impl Iterator<Item = ReceivedRequest> + '_ {
        std::iter::from_fn(|| self.state.received_requests.pop())
    }

    pub fn force_next_response(&self, response: impl IntoResponse) {
        self.state.next_responses.push(response.into_response());
    }
}

async fn graphql_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> axum::response::Response {
    let req = req.into_inner();

    // Record the request incase tests want to inspect it.
    // async_graphql::Request isn't clone so we do a deser roundtrip instead
    state.received_requests.push(ReceivedRequest {
        headers: headers.clone(),
        body: serde_json::from_value(serde_json::to_value(&req).unwrap()).unwrap(),
    });

    if let Some(response) = state.next_responses.pop() {
        return response;
    }

    let headers = headers
        .iter()
        .map(|(name, value)| (name.to_string(), String::from_utf8_lossy(value.as_bytes()).to_string()))
        .collect();

    let response: GraphQLResponse = state.schema.execute(headers, req).await.into();
    response.into_response()
}

#[derive(Clone)]
struct AppState {
    schema: Arc<dyn Schema>,
    received_requests: Arc<crossbeam_queue::SegQueue<ReceivedRequest>>,
    next_responses: Arc<crossbeam_queue::SegQueue<axum::response::Response>>,
}

pub trait Subgraph: 'static {
    fn name(&self) -> String;
    fn start(self) -> impl Future<Output = MockGraphQlServer> + Send;
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

    fn with_sdl(self, sdl: &str) -> SchemaWithSdlOverride
    where
        Self: Sized + 'static,
    {
        SchemaWithSdlOverride {
            schema: Box::new(self),
            sdl: sdl.to_string(),
        }
    }
}

pub struct SchemaWithSdlOverride {
    pub schema: Box<dyn Schema>,
    pub sdl: String,
}

#[async_trait::async_trait]
impl Schema for SchemaWithSdlOverride {
    async fn execute(
        &self,
        headers: Vec<(String, String)>,
        request: async_graphql::Request,
    ) -> async_graphql::Response {
        self.schema.execute(headers, request).await
    }

    fn execute_stream(
        &self,
        request: async_graphql::Request,
    ) -> futures::stream::BoxStream<'static, async_graphql::Response> {
        self.schema.execute_stream(request)
    }

    fn sdl(&self) -> String {
        self.sdl.clone()
    }

    fn with_sdl(self, sdl: &str) -> SchemaWithSdlOverride
    where
        Self: Sized + 'static,
    {
        SchemaWithSdlOverride {
            schema: self.schema,
            sdl: sdl.to_string(),
        }
    }
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
