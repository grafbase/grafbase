use crate::{dev::gateway_nanny::GatewayNanny, ConfigWatcher};
use async_graphql::{EmptySubscription, Schema};
use async_graphql_axum::GraphQL;
use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::{Html, IntoResponse},
    routing::get,
    Json,
};
use common::environment::Environment;
use engine::BatchRequest;
use futures_util::{
    future::{join_all, BoxFuture},
    stream,
};
use gateway_v2::{
    local_server::{WebsocketAccepter, WebsocketService},
    streaming::{encode_stream_response, StreamingFormat},
};
use graphql_composition::FederatedGraph;
use handlebars::Handlebars;
use runtime::context::RequestContext as _;
use serde_json::json;
use std::{net::SocketAddr, time::Duration};
use tokio::sync::{
    mpsc::{self, UnboundedReceiver, UnboundedSender},
    watch,
};
use tower_http::cors::CorsLayer;

use self::{
    batch_response::BatchResponse,
    bus::{AdminBus, ComposeBus, GatewayWatcher, RefreshBus},
    composer::Composer,
    refresher::Refresher,
    ticker::Ticker,
};

mod admin;
mod batch_response;
mod bus;
mod composer;
mod gateway_nanny;
mod refresher;
mod ticker;

const REFRESH_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Clone)]
struct ProxyState {
    admin_pathfinder_html: Html<String>,
    gateway: GatewayWatcher,
}

pub(super) async fn run(
    listen_address: SocketAddr,
    config: ConfigWatcher,
    graph: Option<FederatedGraph>,
) -> Result<(), crate::Error> {
    log::trace!("starting the federated dev server");

    let (gateway_sender, gateway) = watch::channel(gateway_nanny::new_gateway(graph, &config.borrow()));
    let (websocket_sender, websocket_receiver) = mpsc::channel(16);

    let websocket_accepter = WebsocketAccepter::new(websocket_receiver, gateway.clone());
    tokio::spawn(websocket_accepter.handler());

    let admin_schema = if gateway.borrow().is_some() {
        log::debug!("Disabling subgraph composition, federated graph was provided.");
        Schema::build(admin::QueryRoot, admin::MutationRoot, EmptySubscription)
            .data(AdminBus::new_static())
            .finish()
    } else {
        let (graph_sender, graph_receiver) = watch::channel(None);
        let (compose_sender, compose_receiver) = mpsc::channel(16);
        let (refresh_sender, refresh_receiver) = mpsc::channel(16);
        let refresh_bus = RefreshBus::new(refresh_receiver, compose_sender.clone());
        let compose_bus = ComposeBus::new(graph_sender, refresh_sender, compose_sender.clone(), compose_receiver);
        let composer = Composer::new(compose_bus);
        tokio::spawn(composer.handler());

        let ticker = Ticker::new(REFRESH_INTERVAL, compose_sender.clone());
        tokio::spawn(ticker.handler());

        let refresher = Refresher::new(refresh_bus);
        tokio::spawn(refresher.handler());

        let nanny = GatewayNanny::new(graph_receiver, config, gateway_sender);
        tokio::spawn(nanny.handler());

        let admin_bus = AdminBus::new_dynamic(compose_sender);

        Schema::build(admin::QueryRoot, admin::MutationRoot, EmptySubscription)
            .data(admin_bus)
            .finish()
    };

    let environment = Environment::get();
    let static_asset_path = environment.user_dot_grafbase_path.join("static");

    let app = axum::Router::new()
        .route("/admin", get(admin).post_service(GraphQL::new(admin_schema)))
        .route("/graphql", get(engine_get).post(engine_post))
        .route_service("/ws", WebsocketService::new(websocket_sender))
        .nest_service("/static", tower_http::services::ServeDir::new(static_asset_path))
        .layer(CorsLayer::permissive())
        .with_state(ProxyState {
            admin_pathfinder_html: Html(render_pathfinder(listen_address.port(), "/admin")),
            gateway,
        });

    let listener = tokio::net::TcpListener::bind(&listen_address).await.unwrap();
    axum::serve(listener, app)
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
async fn admin(
    State(ProxyState {
        admin_pathfinder_html, ..
    }): State<ProxyState>,
) -> impl IntoResponse {
    admin_pathfinder_html
}

async fn engine_get(
    Query(request): Query<engine::QueryParamRequest>,
    headers: HeaderMap,
    State(ProxyState { gateway, .. }): State<ProxyState>,
) -> impl IntoResponse {
    handle_engine_request(engine::BatchRequest::Single(request.into()), gateway, headers).await
}

async fn engine_post(
    State(ProxyState { gateway, .. }): State<ProxyState>,
    headers: HeaderMap,
    Json(request): Json<engine::BatchRequest>,
) -> impl IntoResponse {
    handle_engine_request(request, gateway, headers).await
}

async fn handle_engine_request(
    request: engine::BatchRequest,
    gateway: GatewayWatcher,
    headers: HeaderMap,
) -> impl IntoResponse {
    log::debug!("engine request received");
    let Some(gateway) = gateway.borrow().clone() else {
        return Json(json!({
            "errors": [{"message": "there are no subgraphs registered currently"}]
        }))
        .into_response();
    };

    let streaming_format = headers
        .get(http::header::ACCEPT)
        .and_then(|value| value.to_str().ok())
        .and_then(StreamingFormat::from_accept_header);

    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();

    let ctx = RequestContext {
        ray_id: ulid::Ulid::new().to_string(),
        headers,
        wait_until_sender: sender,
    };

    let ray_id = ctx.ray_id.clone();

    if matches!(request, BatchRequest::Batch(_)) && streaming_format.is_some() {
        let format = streaming_format.unwrap();

        let (headers, stream) = encode_stream_response(
            ray_id,
            stream::once(async {
                engine_v2::Response::error(
                    "Batch requests cannot be combined with streaming response formats at present",
                    [],
                )
            }),
            format,
        )
        .await;

        return (headers, axum::body::Body::from_stream(stream)).into_response();
    }

    let Some(session) = gateway.authorize(ctx.headers()).await else {
        match (request, streaming_format) {
            (BatchRequest::Single(_), None) => {
                let response = gateway_v2::Response::unauthorized();

                return (response.status, response.headers, response.bytes).into_response();
            }
            (BatchRequest::Single(_), Some(format)) => {
                let (headers, stream) = encode_stream_response(
                    ray_id,
                    stream::once(async { engine_v2::Response::error("Unauthorized", []) }),
                    format,
                )
                .await;

                return (headers, axum::body::Body::from_stream(stream)).into_response();
            }
            (BatchRequest::Batch(requests), _) => {
                let batch_response = BatchResponse::Batch(
                    std::iter::repeat_with(gateway_v2::Response::unauthorized)
                        .take(requests.len())
                        .collect(),
                );

                return batch_response.into_response();
            }
        }
    };

    let response = match (request, streaming_format) {
        (BatchRequest::Single(mut request), None) => {
            request.ray_id = ctx.ray_id.clone();
            BatchResponse::Single(session.execute(&ctx, request).await)
        }
        (BatchRequest::Single(mut request), Some(streaming_format)) => {
            request.ray_id = ctx.ray_id.clone();

            let (headers, stream) =
                encode_stream_response(ray_id, session.execute_stream(request), streaming_format).await;

            tokio::spawn(wait(receiver));

            return (headers, axum::body::Body::from_stream(stream)).into_response();
        }
        (BatchRequest::Batch(requests), None) => {
            let mut responses = Vec::with_capacity(requests.len());
            for mut request in requests {
                request.ray_id = ctx.ray_id.clone();
                responses.push(session.clone().execute(&ctx, request).await)
            }
            BatchResponse::Batch(responses)
        }
        (BatchRequest::Batch(_), Some(_)) => {
            unreachable!("should have been dealt with above")
        }
    };

    tokio::spawn(wait(receiver));

    response.into_response()
}

#[derive(Clone)]
struct RequestContext {
    ray_id: String,
    headers: http::HeaderMap,
    wait_until_sender: UnboundedSender<BoxFuture<'static, ()>>,
}

#[async_trait::async_trait]
impl runtime::context::RequestContext for RequestContext {
    fn ray_id(&self) -> &str {
        &self.ray_id
    }

    async fn wait_until(&self, fut: BoxFuture<'static, ()>) {
        self.wait_until_sender
            .send(fut)
            .expect("Channel is not closed before finishing all wait_until");
    }

    fn headers(&self) -> &http::HeaderMap {
        &self.headers
    }
}

async fn wait(mut receiver: UnboundedReceiver<BoxFuture<'static, ()>>) {
    // Wait simultaneously on everything immediately accessible
    join_all(std::iter::from_fn(|| receiver.try_recv().ok())).await;
    // Wait sequentially on the rest
    while let Some(fut) = receiver.recv().await {
        fut.await;
    }
}
