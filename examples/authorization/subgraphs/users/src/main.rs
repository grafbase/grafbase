use async_graphql::{
    Context, EmptyMutation, EmptySubscription, Object, SDLExportOptions, Schema, SimpleObject, http::GraphiQLSource,
};
use async_graphql_axum::GraphQL;
use axum::{
    Router,
    response::Html,
    routing::{get, post_service},
};
use tokio::{net::TcpListener, signal};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .enable_federation()
        .data(Data::default())
        .finish();

    let sdl = schema.sdl_with_options(SDLExportOptions::new().federation().compose_directive());

    let app = Router::new()
        .route("/graphql", post_service(GraphQL::new(schema)))
        .route("/sdl", get(|| async move { Html(sdl.clone()) }))
        .route(
            "/",
            get(|| async move { Html(GraphiQLSource::build().endpoint("/graphql").finish()) }),
        );

    println!("GraphiQL IDE: http://localhost:4000");
    axum::serve(TcpListener::bind("0.0.0.0:4000").await?, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
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
}

impl Default for Data {
    fn default() -> Self {
        let users = vec![
            User { id: 1, name: "Alice" },
            User { id: 2, name: "Bob" },
            User { id: 3, name: "Musti" },
            User { id: 4, name: "Naukio" },
        ];
        Self { users }
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
    name = "sensible",
    location = "FieldDefinition",
    location = "Object",
    composable = "https://my-spec.com/my-extension/v1.0"
)]
fn sensible(arguments: Option<String>, response: Option<String>) {}

#[derive(Clone, Copy, SimpleObject)]
#[graphql(directive = jwt_scope::apply("user".to_string()))]
pub struct User {
    id: u64,
    name: &'static str,
}

pub struct Query;

#[Object]
impl Query {
    async fn users(&self, ctx: &Context<'_>) -> Vec<Option<User>> {
        ctx.data_unchecked::<Data>().users.iter().copied().map(Some).collect()
    }

    #[graphql(
        directive = sensible::apply(Some("id".to_string()), None)
    )]
    async fn user(&self, ctx: &Context<'_>, id: u64) -> Option<User> {
        ctx.data_unchecked::<Data>()
            .users
            .iter()
            .find(|user| user.id == id)
            .copied()
    }

    #[graphql(entity)]
    async fn find_user_by_id(&self, ctx: &Context<'_>, id: u64) -> User {
        ctx.data_unchecked::<Data>()
            .users
            .iter()
            .find(|user| user.id == id)
            .copied()
            .unwrap()
    }
}
