use axum::{extract::State, response::IntoResponse};
use handlebars::Handlebars;
use serde_json::json;

use super::state::ServerState;

pub(super) fn render(url: &str, graphql_path: &str) -> String {
    let mut handlebars = Handlebars::new();
    let template = include_str!("../../../server/templates/pathfinder.hbs");

    handlebars
        .register_template_string("pathfinder.html", template)
        .expect("must be valid");

    let asset_url = format!("{url}/static");
    let graphql_url = format!("{url}{graphql_path}");

    handlebars
        .render(
            "pathfinder.html",
            &json!({
                "ASSET_URL": asset_url,
                "GRAPHQL_URL": graphql_url,
            }),
        )
        .expect("must render")
}

pub(super) async fn get(
    State(ServerState {
        pathfinder_html: pathfinder,
        ..
    }): State<ServerState>,
) -> impl IntoResponse {
    pathfinder
}
