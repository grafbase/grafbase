use graphql_mocks::FakeGithubSchema;
use integration_tests::{gateway::Gateway, runtime};

#[test]
fn test_cors_allow_origins() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema)
            .with_toml_config(
                r#"
                [cors]
                allow_origins = ["https://app.grafbase.com"]
            "#,
            )
            .build()
            .await;

        // Test allowed origin
        let response = engine
            .raw_execute(
                http::Request::builder()
                    .uri("http://localhost/graphql")
                    .method(http::Method::OPTIONS)
                    .header("Origin", "https://app.grafbase.com")
                    .header("Access-Control-Request-Method", "POST")
                    .body(Vec::new())
                    .unwrap(),
            )
            .await;

        assert_eq!(response.status(), 200);
        assert_eq!(
            response.headers().get("Access-Control-Allow-Origin").unwrap(),
            "https://app.grafbase.com"
        );

        // Test disallowed origin
        let response = engine
            .raw_execute(
                http::Request::builder()
                    .uri("http://localhost/graphql")
                    .method(http::Method::OPTIONS)
                    .header("Origin", "https://example.com")
                    .header("Access-Control-Request-Method", "POST")
                    .body(Vec::new())
                    .unwrap(),
            )
            .await;

        assert_eq!(response.status(), 200);
        assert_eq!(response.headers().get("Access-Control-Allow-Origin"), None);
    });
}

#[test]
fn test_cors_allow_methods() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema)
            .with_toml_config(
                r#"
                [cors]
                allow_origins = ["https://app.grafbase.com"]
                allow_methods = ["GET", "POST"]
            "#,
            )
            .build()
            .await;

        // Test allowed method
        let response = engine
            .raw_execute(
                http::Request::builder()
                    .uri("http://localhost/graphql")
                    .method(http::Method::OPTIONS)
                    .header("Origin", "https://app.grafbase.com")
                    .header("Access-Control-Request-Method", "POST")
                    .body(Vec::new())
                    .unwrap(),
            )
            .await;

        assert_eq!(response.status(), 200);
        assert_eq!(
            response.headers().get("Access-Control-Allow-Methods").unwrap(),
            "GET,POST"
        );

        // Test disallowed method
        let response = engine
            .raw_execute(
                http::Request::builder()
                    .uri("http://localhost/graphql")
                    .method(http::Method::OPTIONS)
                    .header("Origin", "https://app.grafbase.com")
                    .header("Access-Control-Request-Method", "PUT")
                    .body(Vec::new())
                    .unwrap(),
            )
            .await;

        assert_eq!(response.status(), 200);
        assert_eq!(
            response.headers().get("Access-Control-Allow-Methods").unwrap(),
            "GET,POST"
        );
    });
}

#[test]
fn test_cors_allow_headers() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema)
            .with_toml_config(
                r#"
                [cors]
                allow_origins = ["https://app.grafbase.com"]
                allow_methods = ["POST"]
                allow_headers = ["Content-Type"]
            "#,
            )
            .build()
            .await;

        // Test allowed header
        let response = engine
            .raw_execute(
                http::Request::builder()
                    .uri("http://localhost/graphql")
                    .method(http::Method::OPTIONS)
                    .header("Origin", "https://app.grafbase.com")
                    .header("Access-Control-Request-Method", "POST")
                    .header("Access-Control-Request-Headers", "content-type")
                    .body(Vec::new())
                    .unwrap(),
            )
            .await;

        assert_eq!(response.status(), 200);
        assert_eq!(
            response.headers().get("Access-Control-Allow-Headers").unwrap(),
            "content-type"
        );

        // Test disallowed header
        let response = engine
            .raw_execute(
                http::Request::builder()
                    .uri("http://localhost/graphql")
                    .method(http::Method::OPTIONS)
                    .header("Origin", "https://app.grafbase.com")
                    .header("Access-Control-Request-Method", "POST")
                    .header("Access-Control-Request-Headers", "X-Custom-Header")
                    .body(Vec::new())
                    .unwrap(),
            )
            .await;

        assert_eq!(response.status(), 200);
        assert_eq!(
            response.headers().get("Access-Control-Allow-Headers").unwrap(),
            "content-type"
        );
    });
}

#[test]
fn test_cors_credentials() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema)
            .with_toml_config(
                r#"
                [cors]
                allow_origins = ["https://app.grafbase.com"]
                allow_credentials = true
            "#,
            )
            .build()
            .await;

        let response = engine
            .raw_execute(
                http::Request::builder()
                    .uri("http://localhost/graphql")
                    .method(http::Method::OPTIONS)
                    .header("Origin", "https://app.grafbase.com")
                    .header("Access-Control-Request-Method", "POST")
                    .body(Vec::new())
                    .unwrap(),
            )
            .await;

        assert_eq!(response.status(), 200);
        assert_eq!(
            response.headers().get("Access-Control-Allow-Credentials").unwrap(),
            "true"
        );
    });
}

#[test]
fn test_cors_max_age() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema)
            .with_toml_config(
                r#"
                [cors]
                allow_origins = ["https://app.grafbase.com"]
                max_age = "60s"
            "#,
            )
            .build()
            .await;

        let response = engine
            .raw_execute(
                http::Request::builder()
                    .uri("http://localhost/graphql")
                    .method(http::Method::OPTIONS)
                    .header("Origin", "https://app.grafbase.com")
                    .header("Access-Control-Request-Method", "POST")
                    .body(Vec::new())
                    .unwrap(),
            )
            .await;

        assert_eq!(response.status(), 200);
        assert_eq!(response.headers().get("Access-Control-Max-Age").unwrap(), "60");
    });
}

#[test]
fn test_cors_expose_headers() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema)
            .with_toml_config(
                r#"
                [cors]
                allow_origins = ["https://app.grafbase.com"]
                expose_headers = ["Content-Encoding"]
            "#,
            )
            .build()
            .await;

        let response = engine.post("{ __typename }").await;

        assert_eq!(response.status, 200);
        assert_eq!(
            response.headers.get("Access-Control-Expose-Headers").unwrap(),
            "content-encoding"
        );
    });
}

#[test]
fn mcp() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema)
            .with_toml_config(
                r#"
                [cors]
                allow_origins = ["https://app.grafbase.com"]
                expose_headers = ["Content-Encoding"]
            "#,
            )
            .build()
            .await;

        let response = engine
            .raw_execute(
                http::Request::builder()
                    .uri("http://localhost/mcp")
                    .method(http::Method::OPTIONS)
                    .header("Origin", "https://app.grafbase.com")
                    .header("Access-Control-Request-Method", "POST")
                    .body(Vec::new())
                    .unwrap(),
            )
            .await;

        assert_eq!(response.status(), 200);
        assert_eq!(
            response.headers().get("Access-Control-Allow-Origin").unwrap(),
            "https://app.grafbase.com"
        );
    });
}
