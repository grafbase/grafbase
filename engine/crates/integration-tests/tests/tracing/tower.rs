use std::time::Duration;

use axum::routing::get;
use axum::Router;
use tracing::Level;
use tracing_mock::{expect, subscriber};

use grafbase_telemetry::span::request::GATEWAY_SPAN_NAME;

// when using the tower layer there should be a span named gateway
#[tokio::test(flavor = "current_thread")]
async fn expect_gateway_span() {
    // the span we're expecting
    let span = expect::span().at_level(Level::INFO).named(GATEWAY_SPAN_NAME);

    // subscriber expectations
    let (subscriber, handle) = subscriber::mock()
        .with_filter(|meta| meta.is_span() && meta.target() == "grafbase" && *meta.level() >= Level::INFO)
        .new_span(span)
        .run_with_handle();

    // set the global default
    let _default = tracing::subscriber::set_default(subscriber);

    // axum server with our tower layer
    let app = Router::new()
        .route("/", get(|| async {}))
        .layer(grafbase_telemetry::tower::layer(
            grafbase_telemetry::metrics::meter_from_global_provider(),
        ));

    let tcp_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = tcp_listener.local_addr().unwrap().port();

    tokio::spawn(async {
        axum::serve(tcp_listener, app.into_make_service()).await.unwrap();
    });

    // let it spin up
    tokio::time::sleep(Duration::from_millis(30)).await;

    // issue a request to generate the intended span
    let response = reqwest::get(format!("http://127.0.0.1:{port}")).await.unwrap();

    // assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);

    handle.assert_finished();
}
