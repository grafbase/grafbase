use async_graphql::{http::GraphiQLSource, EmptyMutation, EmptySubscription, Schema};
use async_graphql_axum::GraphQL;
use axum::{
    response::{self, IntoResponse},
    routing::{get, post_service},
    Router,
};
use schema::{QueryRoot, Users};
use tokio::net::TcpListener;

mod schema;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .enable_federation()
        .data(Users::new())
        .finish();

    let app = Router::new()
        .route("/graphql", post_service(GraphQL::new(schema)))
        .route("/", get(graphiql));

    println!("GraphiQL IDE: http://localhost:4000");
    axum::serve(TcpListener::bind("127.0.0.1:4000").await?, app).await?;

    Ok(())
}

async fn graphiql() -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint("/graphql").finish())
}
