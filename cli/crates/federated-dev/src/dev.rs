use std::time::Duration;

use async_graphql::{EmptySubscription, Schema};
use async_graphql_axum::GraphQL;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::get,
    Server,
};
use handlebars::Handlebars;
use serde_json::json;
use tokio::sync::mpsc;

use self::{
    bus::{AdminBus, ComposeBus, RefreshBus},
    composer::Composer,
    refresher::Refresher,
    router::Router,
    ticker::Ticker,
};

mod admin;
mod bus;
mod composer;
mod refresher;
mod router;
mod ticker;

const REFRESH_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Clone)]
struct ProxyState {
    pathfinder_html: Html<String>,
    admin_pathfinder_html: Html<String>,
}

pub(super) async fn run(port: u16) -> Result<(), crate::Error> {
    let (graph_sender, graph_receiver) = mpsc::channel(16);
    let (refresh_sender, refresh_receiver) = mpsc::channel(16);
    let (compose_sender, compose_receiver) = mpsc::channel(16);

    let compose_bus = ComposeBus::new(graph_sender, refresh_sender, compose_sender.clone(), compose_receiver);
    let refresh_bus = RefreshBus::new(refresh_receiver, compose_sender.clone());
    let admin_bus = AdminBus::new(compose_sender.clone());

    let composer = Composer::new(compose_bus);
    tokio::spawn(composer.handler());

    let refresher = Refresher::new(refresh_bus);
    tokio::spawn(refresher.handler());

    let router = Router::new(graph_receiver);
    tokio::spawn(router.handler());

    let ticker = Ticker::new(REFRESH_INTERVAL, compose_sender);
    tokio::spawn(ticker.handler());

    let schema = Schema::build(admin::QueryRoot, admin::MutationRoot, EmptySubscription)
        .data(admin_bus)
        .finish();

    let app = axum::Router::new()
        .route("/", get(root))
        .route("/admin", get(admin).post_service(GraphQL::new(schema)))
        .with_state(ProxyState {
            pathfinder_html: Html(render_pathfinder(port, "/graphql")),
            admin_pathfinder_html: Html(render_pathfinder(port, "/admin")),
        });

    let host = format!("127.0.0.1:{port}");
    let address = host.parse().expect("we just defined it above, it _must work_");

    Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .map_err(|error| crate::Error::internal(error.to_string()))?;

    Ok(())
}

fn render_pathfinder(port: u16, graphql_url: &str) -> String {
    let mut handlebars = Handlebars::new();
    let template = include_str!("../../server/templates/pathfinder.hbs");

    handlebars
        .register_template_string("pathfinder.html", template)
        .expect("must be valid");

    let asset_url = format!("http://127.0.0.1:{port}/static");

    handlebars
        .render(
            "pathfinder.html",
            &json!({
                "ASSET_URL": asset_url,
                "GRAPHQL_URL": graphql_url
            }),
        )
        .expect("must render")
}

#[allow(clippy::unused_async)]
async fn root(State(ProxyState { pathfinder_html, .. }): State<ProxyState>) -> impl IntoResponse {
    pathfinder_html
}

#[allow(clippy::unused_async)]
async fn admin(
    State(ProxyState {
        admin_pathfinder_html, ..
    }): State<ProxyState>,
) -> impl IntoResponse {
    admin_pathfinder_html
}
