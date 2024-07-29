use async_graphql::{
    http::GraphiQLSource, EmptyMutation, EmptySubscription, SDLExportOptions, Schema,
};
use async_graphql_axum::GraphQL;
use axum::{
    response::{self, IntoResponse},
    routing::{get, post_service},
    Router,
};
use schema::Query;
use tokio::net::TcpListener;

mod schema;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let schema = Schema::build(Query::default(), EmptyMutation, EmptySubscription)
        .enable_federation()
        .finish();

    let sdl = schema.sdl_with_options(SDLExportOptions::new().federation().compose_directive());

    let app = Router::new()
        .route("/graphql", post_service(GraphQL::new(schema)))
        .route("/sdl", get(|| async move { response::Html(sdl.clone()) }))
        .route("/", get(graphiql));

    println!("GraphiQL IDE: http://localhost:4000");
    axum::serve(TcpListener::bind("127.0.0.1:4000").await?, app).await?;

    Ok(())
}

async fn graphiql() -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint("/graphql").finish())
}
