use std::{net::TcpListener, sync::atomic::Ordering, time::Duration};

use axum::{
    body::Body,
    extract::State,
    response::{Html, IntoResponse},
    routing::{get, head, post},
    Router,
};
use common::environment::Environment;
use handlebars::Handlebars;
use hyper::{Request, StatusCode};
use serde_json::json;
use tokio::{
    task::{JoinError, JoinSet},
    time::sleep,
};
use tower_http::{cors::CorsLayer, services::ServeDir};

use crate::{atomics::WORKER_PORT, errors::ServerError, servers::PortSelection};

type Client = hyper_util::client::legacy::Client<hyper_util::client::legacy::connect::HttpConnector, Body>;

#[derive(Clone)]
struct ProxyState {
    pathfinder_html: Html<String>,
    client: Client,
}

pub struct ProxyHandle {
    pub port: u16,
    set: JoinSet<Result<(), ServerError>>,
}

pub async fn start(port: PortSelection) -> Result<ProxyHandle, ServerError> {
    let listener = port.into_listener().await?;
    let port = listener.local_addr().expect("must have a local addr").port();
    let mut set = JoinSet::new();
    set.spawn(start_inner(listener));

    Ok(ProxyHandle { port, set })
}

async fn start_inner(listener: TcpListener) -> Result<(), ServerError> {
    // FIXME: Migrate to the new abstractions.
    use hyper_util::client::legacy::Client as HyperClient;
    use hyper_util::rt::TokioExecutor;

    let port = listener.local_addr().expect("must have a local addr").port();
    trace!("starting pathfinder at port {port}");

    let client = HyperClient::builder(TokioExecutor::new())
        .http1_preserve_header_case(true)
        .build_http();

    let mut handlebars = Handlebars::new();
    let template = include_str!("../templates/pathfinder.hbs");
    handlebars
        .register_template_string("pathfinder.html", template)
        .expect("must be valid");
    let proxied_graphql_url = "/graphql".to_string();
    let asset_url = format!("http://127.0.0.1:{port}/static");
    let pathfinder_html = handlebars
        .render(
            "pathfinder.html",
            &json!({
                "ASSET_URL": asset_url,
                "GRAPHQL_URL": proxied_graphql_url
            }),
        )
        .expect("must render");

    let environment = Environment::get();
    let static_asset_path = environment.user_dot_grafbase_path.join("static");

    let router = Router::new()
        .route("/", get(root))
        .route("/graphql", get(graphql))
        .route("/graphql", head(graphql))
        .route("/graphql", post(graphql))
        .route("/admin", get(admin))
        .route("/admin", head(admin))
        .route("/admin", post(admin))
        .nest_service("/static", ServeDir::new(static_asset_path))
        .layer(CorsLayer::permissive())
        .with_state(ProxyState {
            pathfinder_html: Html(pathfinder_html),
            client,
        });

    axum::serve(
        tokio::net::TcpListener::from_std(listener).map_err(ServerError::StartProxyServer)?,
        router,
    )
    // FIXME: Bring back!
    // .preserve_header_case(true)
    .await
    .map_err(ServerError::StartProxyServer)?;

    Ok(())
}

#[allow(clippy::unused_async)]
async fn root(State(ProxyState { pathfinder_html, .. }): State<ProxyState>) -> impl IntoResponse {
    pathfinder_html
}

const POLL_INTERVAL: Duration = Duration::from_millis(200);

async fn graphql(
    State(ProxyState { client, .. }): State<ProxyState>,
    req: Request<Body>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    graphql_inner(client, req, "graphql").await
}

async fn admin(
    State(ProxyState { client, .. }): State<ProxyState>,
    req: Request<Body>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    graphql_inner(client, req, "admin").await
}

async fn graphql_inner(
    client: Client,
    mut req: Request<Body>,
    path: &str,
) -> Result<impl IntoResponse, impl IntoResponse> {
    // Request body size limit for Cloudflare Workers enterprise.
    // See https://developers.cloudflare.com/workers/platform/limits/.
    const REQUEST_BODY_SIZE_LIMIT: usize = 1_024 * 1_024 * 512;

    let query = req.uri().query().map_or(String::new(), |query| format!("?{query}"));

    // http::Request can't be cloned
    let (parts, body) = req.into_parts();
    let body_bytes = axum::body::to_bytes(body, REQUEST_BODY_SIZE_LIMIT)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let body = String::from_utf8(body_bytes.clone().into()).map_err(|_| StatusCode::BAD_REQUEST)?;
    req = Request::from_parts(parts, body_bytes.into());

    loop {
        let worker_port = WORKER_PORT.load(Ordering::Relaxed);

        if worker_port == 0 {
            sleep(POLL_INTERVAL).await;
            continue;
        }

        let uri = format!("http://127.0.0.1:{worker_port}/{path}{query}");

        // http::Request can't be cloned
        let mut cloned_request = Request::builder()
            .method(req.method().clone())
            .uri(uri)
            .version(req.version());

        for header in req.headers() {
            cloned_request = cloned_request.header(header.0, header.1);
        }

        let request = cloned_request
            .body(body.clone().into())
            .expect("must succeed, using an existing request");

        let response = client.request(request).await;

        match response {
            Ok(response) => {
                return Ok(response);
            }
            Err(error) => {
                if error.is_connect() {
                    sleep(POLL_INTERVAL).await;
                } else {
                    return Err(axum::http::StatusCode::BAD_REQUEST);
                }
            }
        };
    }
}

impl ProxyHandle {
    pub async fn join(&mut self) -> Option<Result<Result<(), ServerError>, JoinError>> {
        self.set.join_next().await
    }
}
