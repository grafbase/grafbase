use std::{net::SocketAddr, sync::Arc};

use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{extract::State, response::IntoResponse, routing::post, Router};

use super::DynamicSchema;

pub struct MockGraphQlServer {
    shutdown: Option<tokio::sync::oneshot::Sender<()>>,
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
    schema: Arc<DynamicSchema>,
}

impl MockGraphQlServer {
    pub fn new(schema: Arc<DynamicSchema>, listen_address: SocketAddr) -> Self {
        let state = AppState { schema };

        let app = Router::new()
            .route("/", post(graphql_handler))
            .with_state(state.clone());

        let (shutdown_sender, shutdown_receiver) = tokio::sync::oneshot::channel::<()>();

        tokio::spawn(async move {
            let listener = tokio::net::TcpListener::bind(listen_address).await.unwrap();

            axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    shutdown_receiver.await.ok();
                })
                .await
                .unwrap();
        });

        MockGraphQlServer {
            shutdown: Some(shutdown_sender),
        }
    }
}

async fn graphql_handler(State(state): State<AppState>, req: GraphQLRequest) -> axum::response::Response {
    let req = req.into_inner();
    let response: GraphQLResponse = state.schema.execute(req).await.into();

    response.into_response()
}
