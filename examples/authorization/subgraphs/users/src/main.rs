use std::sync::Arc;

use async_graphql::{
    Context, EmptyMutation, EmptySubscription, Object, SDLExportOptions, Schema, SimpleObject, http::GraphiQLSource,
};
use axum::{
    Json, Router,
    extract::State,
    http::HeaderMap,
    response::{Html, IntoResponse},
    routing::{get, post},
};
use tokio::{net::TcpListener, signal};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt as _, util::SubscriberInitExt as _};

type UsersSchema = Arc<Schema<Query, EmptyMutation, EmptySubscription>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let log_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_ansi(true)
        .with_target(true);

    tracing_subscriber::registry()
        .with(log_layer)
        .with(EnvFilter::new("info"))
        .init();

    let schema: UsersSchema = Arc::new(
        Schema::build(Query, EmptyMutation, EmptySubscription)
            .enable_federation()
            .data(Data::default())
            .finish(),
    );

    let sdl = schema.sdl_with_options(SDLExportOptions::new().federation().compose_directive());

    let app = Router::new()
        .route("/graphql", post(gql))
        .with_state(schema)
        .route("/sdl", get(|| async move { Html(sdl.clone()) }))
        .route(
            "/",
            get(|| async move { Html(GraphiQLSource::build().endpoint("/graphql").finish()) }),
        );

    tracing::info!("GraphiQL IDE: http://localhost:4000");
    axum::serve(TcpListener::bind("0.0.0.0:4000").await?, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn gql(
    State(schema): State<UsersSchema>,
    headers: HeaderMap,
    Json(request): Json<async_graphql::Request>,
) -> impl IntoResponse {
    let response = schema.execute(request.data(headers)).await;
    Json(response)
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    println!("Shutting down gracefully...");
}

struct Data {
    users: Vec<User>,
    accounts: Vec<Account>,
}

impl Default for Data {
    fn default() -> Self {
        let users = vec![
            User { id: 1, name: "Alice" },
            User { id: 2, name: "Bob" },
            User { id: 3, name: "Musti" },
            User { id: 4, name: "Naukio" },
        ];
        let accounts = vec![
            Account {
                id: 1,
                name: "Alice's account",
            },
            Account {
                id: 2,
                name: "Bob's account",
            },
            Account {
                id: 3,
                name: "Musti's account",
            },
            Account {
                id: 4,
                name: "Naukio's account",
            },
        ];
        Self { users, accounts }
    }
}

////////////////////
// GraphQL Schema //
////////////////////

#[async_graphql::TypeDirective(
    name = "jwtScope",
    location = "FieldDefinition",
    location = "Object",
    composable = "https://my-spec.com/my-extension/v1.0"
)]
fn jwt_scope(scope: String) {}

#[async_graphql::TypeDirective(
    name = "accessControl",
    location = "FieldDefinition",
    location = "Object",
    composable = "https://my-spec.com/my-extension/v1.0"
)]
fn access_control(arguments: Option<String>, fields: Option<String>) {}

#[derive(Clone, Copy, SimpleObject)]
#[graphql(directive = jwt_scope::apply("user".to_string()))]
pub struct User {
    id: u64,
    name: &'static str,
}

#[derive(Clone, Copy, SimpleObject)]
#[graphql(directive = jwt_scope::apply("account".to_string()))]
#[graphql(directive = access_control::apply(None, Some("id".to_string())))]
pub struct Account {
    id: u64,
    name: &'static str,
}

pub struct Query;

#[Object]
impl Query {
    async fn accounts(&self, ctx: &Context<'_>) -> Option<async_graphql::FieldResult<Vec<Option<Account>>>> {
        if !ctx
            .data_unchecked::<HeaderMap>()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .contains("account")
        {
            return Some(Err("Insufficient scopes".into()));
        }
        Some(Ok(ctx
            .data_unchecked::<Data>()
            .accounts
            .iter()
            .copied()
            .map(Some)
            .collect()))
    }

    #[graphql(
        directive = access_control::apply(Some("id".to_string()), None)
    )]
    async fn user(&self, ctx: &Context<'_>, id: u64) -> Option<async_graphql::FieldResult<User>> {
        if !ctx
            .data_unchecked::<HeaderMap>()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .contains("user")
        {
            return Some(Err("Insufficient scopes".into()));
        }
        ctx.data_unchecked::<Data>()
            .users
            .iter()
            .find(|user| user.id == id)
            .copied()
            .map(Ok)
    }
}
