//! A mock GraphQL server for testing the GraphQL connector

use std::{net::TcpListener, sync::Arc, time::Duration};

use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{extract::State, http::HeaderMap, routing::post, Router};

mod almost_empty;
mod echo;
mod fake_github;
mod federation;
mod state_mutation;

pub use {
    almost_empty::AlmostEmptySchema,
    echo::EchoSchema,
    fake_github::FakeGithubSchema,
    federation::{FakeFederationAccountsSchema, FakeFederationProductsSchema, FakeFederationReviewsSchema},
    state_mutation::StateMutationSchema,
};

pub struct MockGraphQlServer {
    pub schema: Arc<dyn Schema>,
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
        let state = AppState { schema: schema.clone() };
        let app = Router::new().route("/", post(graphql_handler)).with_state(state);

        let socket = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = socket.local_addr().unwrap().port();

        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

        tokio::spawn(async move {
            axum::Server::from_tcp(socket)
                .unwrap()
                .serve(app.with_state(()).into_make_service())
                .with_graceful_shutdown(async move {
                    shutdown_rx.await.ok();
                })
                .await
                .unwrap();
        });

        // Give the server time to start
        tokio::time::sleep(Duration::from_millis(20)).await;

        MockGraphQlServer {
            schema,
            shutdown: Some(shutdown_tx),
            port,
        }
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

async fn graphql_handler(State(state): State<AppState>, headers: HeaderMap, req: GraphQLRequest) -> GraphQLResponse {
    let headers = headers
        .into_iter()
        .map(|(name, value)| {
            (
                name.map(|name| name.to_string()).unwrap_or_default(),
                String::from_utf8_lossy(value.as_bytes()).to_string(),
            )
        })
        .collect();

    let response: GraphQLResponse = state.schema.execute(headers, req.into_inner()).await.into();
    response
}

#[derive(Clone)]
struct AppState {
    schema: Arc<dyn Schema>,
}

/// Creating a trait for schema so we can use it as a trait object and avoid
/// making everything generic over Query, Mutation & Subscription params
#[async_trait::async_trait]
pub trait Schema: Send + Sync {
    async fn execute(&self, headers: Vec<(String, String)>, request: async_graphql::Request)
        -> async_graphql::Response;

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

    fn sdl(&self) -> String {
        self.sdl_with_options(async_graphql::SDLExportOptions::new())
    }
}
