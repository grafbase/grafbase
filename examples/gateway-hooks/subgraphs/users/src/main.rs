use async_graphql::{
    http::GraphiQLSource, Any, Context, EmptyMutation, EmptySubscription, Name, Object, SDLExportOptions, Schema,
    SimpleObject, Value,
};
use async_graphql_axum::GraphQL;
use axum::{
    response::Html,
    routing::{get, post_service},
    Router,
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
            User {
                id: 1,
                name: "Alice",
                address: Some(Address { street: "123 Folsom" }),
            },
            User {
                id: 2,
                name: "Bob",
                address: Some(Address { street: "123 Castro" }),
            },
            User {
                id: 3,
                name: "Musti",
                address: Some(Address { street: "123 Planet" }),
            },
            User {
                id: 4,
                name: "Naukio",
                address: Some(Address { street: "123 Rocket" }),
            },
        ];
        Self { users }
    }
}

////////////////////
// GraphQL Schema //
////////////////////

#[async_graphql::TypeDirective(
    name = "authorized",
    location = "FieldDefinition",
    location = "Object",
    composable = "https://custom.spec.dev/extension/v1.0"
)]
fn authorized(arguments: Option<String>, fields: Option<String>, node: Option<String>, metadata: Option<Any>) {}

#[derive(Clone, Copy, SimpleObject)]
pub struct User {
    id: u64,
    name: &'static str,
    // @authorized(fields: "id")
    #[graphql(
        directive = authorized::apply(None, Some("id".to_string()), None, None)
    )]
    address: Option<Address>,
}

#[derive(Clone, Copy, SimpleObject)]
pub struct Address {
    street: &'static str,
}

pub struct Query;

#[Object]
impl Query {
    // @authorized(node: "id", metadata: { role: "admin "})
    #[graphql(
        directive = authorized::apply(None, None, Some("id".to_string()), Some(Any(Value::Object([(Name::new("role"), "admin".into())].into()))))
    )]
    async fn users(&self, ctx: &Context<'_>) -> Vec<Option<User>> {
        ctx.data_unchecked::<Data>().users.iter().copied().map(Some).collect()
    }

    // @authorized(arguments: "id")
    #[graphql(
        directive = authorized::apply(Some("id".to_string()), None, None, None),
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
