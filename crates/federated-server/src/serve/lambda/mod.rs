mod layer;

pub(super) async fn bind(path: &str, router: axum::Router<()>, mcp_url: Option<String>) -> crate::Result<()> {
    let app = tower::ServiceBuilder::new()
        .layer(layer::LambdaLayer::default())
        .service(router);

    tracing::info!("GraphQL endpoint exposed at {path}");

    if let Some(mcp_url) = mcp_url {
        tracing::info!("MCP endpoint exposed at {mcp_url}");
    }

    lambda_http::run(app).await.expect("cannot start lambda http server");

    Ok(())
}
