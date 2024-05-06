use axum::{routing::get, Router};
use engine::registry::resolvers::http::HttpResolver;
use tower_service::Service;
use worker::*;

fn router() -> Router {
    Router::new().route("/", get(root))
}

#[event(fetch)]
async fn fetch(req: HttpRequest, _env: Env, _ctx: Context) -> Result<axum::http::Response<axum::body::Body>> {
    console_error_panic_hook::set_once();
    Ok(router().call(req).await?)
}

pub async fn root() -> &'static str {
    HttpResolver {
        method: "GET".to_owned(),
        url: todo!(),
        api_name: todo!(),
        path_parameters: todo!(),
        query_parameters: todo!(),
        request_body: todo!(),
        expected_status: todo!(),
    }
    .resolve()
    .await
    .unwrap()
}
