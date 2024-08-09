use async_graphql::{
    http::GraphiQLSource, Any, ComplexObject, Context, EmptyMutation, EmptySubscription, Object, SDLExportOptions,
    Schema, SimpleObject,
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

    println!("GraphiQL IDE: http://localhost:4002");
    axum::serve(TcpListener::bind("0.0.0.0:4002").await?, app)
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
    pets: Vec<Pet>,
}

impl Default for Data {
    fn default() -> Self {
        // generate list of pets dogs and cats
        let pets = vec![
            Pet { id: 1, name: "Rex" },
            Pet { id: 2, name: "Mittens" },
            Pet { id: 3, name: "Fluffy" },
            Pet { id: 4, name: "Spot" },
        ];

        Self { pets }
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
#[graphql(complex)]
pub struct Pet {
    id: u64,
    name: &'static str,
}

#[ComplexObject]
impl Pet {
    async fn age(&self) -> String {
        let mut age = time::Duration::days((rand::random::<f64>() * 365.0 * 10.0) as i64);
        let years = age.whole_days() / 365;
        age -= time::Duration::days(years * 365);
        let months = age.whole_days() / 31;
        age -= time::Duration::days(months * 31);
        let days = age.whole_days();

        format!("{years}years, {months}months, {days}days")
    }
}

#[derive(SimpleObject)]
#[graphql(complex)]
struct User {
    id: u64,
}

#[ComplexObject]
impl User {
    async fn pets(&self, ctx: &Context<'_>) -> Vec<Pet> {
        let data = ctx.data_unchecked::<Data>();
        match self.id {
            1 => vec![data.pets[0], data.pets[1]],
            2 => vec![data.pets[2]],
            3 => vec![data.pets[3]],
            _ => vec![],
        }
    }
}

pub struct Query;

#[Object]
impl Query {
    async fn pets(&self, ctx: &Context<'_>) -> Vec<Option<Pet>> {
        ctx.data_unchecked::<Data>().pets.iter().copied().map(Some).collect()
    }

    #[graphql(entity)]
    async fn find_pet_by_id(&self, ctx: &Context<'_>, id: u64) -> Option<Pet> {
        ctx.data_unchecked::<Data>()
            .pets
            .iter()
            .find(|pet| pet.id == id)
            .copied()
    }

    #[graphql(entity)]
    async fn find_user_by_id(&self, id: u64) -> User {
        User { id }
    }
}
