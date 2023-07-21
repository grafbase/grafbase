#![allow(unused)]

use crate::errors::ServerError;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use common::environment::Environment;
use handlebars::Handlebars;
use serde_json::json;
use std::net::{Ipv4Addr, SocketAddr};
use tower_http::services::ServeDir;

pub async fn serve(port: u16, worker_port: u16) -> Result<(), ServerError> {
    let mut handlebars = Handlebars::new();
    let template = include_str!("../templates/playground.html");
    handlebars
        .register_template_string("playground.html", template)
        .expect("must be valid");
    let graphql_url = format!("http://127.0.0.1:{worker_port}/graphql");
    let playground_html = handlebars
        .render(
            "playground.html",
            &json!({
                "ASSET_URL": "/static",
                "GRAPHQL_URL": graphql_url
            }),
        )
        .expect("must render");

    let environment = Environment::get();
    let static_asset_path = environment.user_dot_grafbase_path.join("static");

    let router = Router::new()
        .route("/", get(root))
        .nest_service("/static", ServeDir::new(static_asset_path))
        .with_state(Html(playground_html));

    // TODO change this to `Ipv6Addr::UNSPECIFIED`
    // if we upgrade to miniflare 3 / stop using miniflare
    axum::Server::bind(&SocketAddr::from((Ipv4Addr::LOCALHOST, port)))
        .serve(router.into_make_service())
        .await
        // FIXME
        .map_err(ServerError::ChangeMe);

    Ok(())

    // TODO handle codicon download
}

#[allow(clippy::unused_async)]
async fn root(State(playground_html): State<Html<String>>) -> impl IntoResponse {
    playground_html
}

#[tokio::test]
async fn test() {
    Environment::try_init(None).unwrap();
    serve(3030, 4000).await;
}
