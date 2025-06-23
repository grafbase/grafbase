use std::{
    net::{Ipv4Addr, SocketAddrV4},
    sync::Arc,
};

use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{Router, extract::State, response::IntoResponse, routing::post};
use url::Url;

/// Represents a mock GraphQL server used for testing purposes.
pub struct MockGraphQlServer {
    shutdown: Option<tokio::sync::oneshot::Sender<()>>,
    state: AppState,
    url: Url,
    name: String,
}

impl Drop for MockGraphQlServer {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            shutdown.send(()).ok();
        }
    }
}

#[derive(Clone)]
struct AppState {
    schema: Arc<(async_graphql::dynamic::Schema, String)>,
}

impl MockGraphQlServer {
    pub(crate) async fn new(name: String, schema: Arc<(async_graphql::dynamic::Schema, String)>) -> Self {
        let state = AppState { schema };

        let app = Router::new()
            .route("/", post(graphql_handler))
            .with_state(state.clone());

        let (shutdown_sender, shutdown_receiver) = tokio::sync::oneshot::channel::<()>();

        let listen_address = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 0);
        let listener = tokio::net::TcpListener::bind(listen_address).await.unwrap();

        let listen_address = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    shutdown_receiver.await.ok();
                })
                .await
                .unwrap();
        });

        let url = format!("http://{listen_address}").parse().unwrap();

        MockGraphQlServer {
            shutdown: Some(shutdown_sender),
            url,
            state,
            name,
        }
    }

    /// Returns a reference to the URL of the mock GraphQL server
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Returns the GraphQL schema in SDL (Schema Definition Language)
    pub fn schema(&self) -> &str {
        &self.state.schema.1
    }

    /// Returns the name of the subgraph
    pub fn name(&self) -> &str {
        &self.name
    }
}

async fn graphql_handler(State(state): State<AppState>, req: GraphQLRequest) -> axum::response::Response {
    let req = req.into_inner();
    let response: GraphQLResponse = state.schema.0.execute(req).await.into();

    response.into_response()
}
