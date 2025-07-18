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
fn test_cors_allow_origins_single_value() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema)
            .with_toml_config(
                r#"
                [cors]
                allow_origins = "https://app.grafbase.com"
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

                [mcp]
                enabled = true
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

#[test]
fn test_cors_wildcard_subdomain_pattern() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema)
            .with_toml_config(
                r#"
                [cors]
                allow_origins = ["https://*.grafbase.com"]
            "#,
            )
            .build()
            .await;

        // Test allowed wildcard subdomain
        let response = engine
            .raw_execute(
                http::Request::builder()
                    .uri("http://localhost/graphql")
                    .method(http::Method::OPTIONS)
                    .header("Origin", "https://dev.grafbase.com")
                    .header("Access-Control-Request-Method", "POST")
                    .body(Vec::new())
                    .unwrap(),
            )
            .await;

        assert_eq!(response.status(), 200);
        assert_eq!(
            response.headers().get("Access-Control-Allow-Origin").unwrap(),
            "https://dev.grafbase.com"
        );

        // Only matches subdomains
        let response = engine
            .raw_execute(
                http::Request::builder()
                    .uri("http://localhost/graphql")
                    .method(http::Method::OPTIONS)
                    .header("Origin", "https://grafbase.com")
                    .header("Access-Control-Request-Method", "POST")
                    .body(Vec::new())
                    .unwrap(),
            )
            .await;

        assert_eq!(response.status(), 200);
        assert_eq!(response.headers().get("Access-Control-Allow-Origin"), None);

        // Test disallowed domain
        let response = engine
            .raw_execute(
                http::Request::builder()
                    .uri("http://localhost/graphql")
                    .method(http::Method::OPTIONS)
                    .header("Origin", "https://malicious.com")
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
fn test_cors_mixed_static_and_wildcard_origins() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema)
            .with_toml_config(
                r#"
                [cors]
                allow_origins = [
                    "https://*.grafbase.com",
                    "https://test.com",
                ]
            "#,
            )
            .build()
            .await;

        // Test wildcard pattern
        let response = engine
            .raw_execute(
                http::Request::builder()
                    .uri("http://localhost/graphql")
                    .method(http::Method::OPTIONS)
                    .header("Origin", "https://dev.grafbase.com")
                    .header("Access-Control-Request-Method", "POST")
                    .body(Vec::new())
                    .unwrap(),
            )
            .await;

        assert_eq!(response.status(), 200);
        assert_eq!(
            response.headers().get("Access-Control-Allow-Origin").unwrap(),
            "https://dev.grafbase.com"
        );

        // Test static origin
        let response = engine
            .raw_execute(
                http::Request::builder()
                    .uri("http://localhost/graphql")
                    .method(http::Method::OPTIONS)
                    .header("Origin", "https://test.com")
                    .header("Access-Control-Request-Method", "POST")
                    .body(Vec::new())
                    .unwrap(),
            )
            .await;

        assert_eq!(response.status(), 200);
        assert_eq!(
            response.headers().get("Access-Control-Allow-Origin").unwrap(),
            "https://test.com"
        );

        let response = engine
            .raw_execute(
                http::Request::builder()
                    .uri("http://localhost/graphql")
                    .method(http::Method::OPTIONS)
                    .header("Origin", "https://malicious.com")
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
fn test_cors_multiple_wildcard_origins() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema)
            .with_toml_config(
                r#"
                [cors]
                allow_origins = [
                    "https://*.grafbase.com",
                    "https://*.example.com",
                ]
            "#,
            )
            .build()
            .await;

        // Test first wildcard domain
        let response = engine
            .raw_execute(
                http::Request::builder()
                    .uri("http://localhost/graphql")
                    .method(http::Method::OPTIONS)
                    .header("Origin", "https://api.grafbase.com")
                    .header("Access-Control-Request-Method", "POST")
                    .body(Vec::new())
                    .unwrap(),
            )
            .await;

        assert_eq!(response.status(), 200);
        assert_eq!(
            response.headers().get("Access-Control-Allow-Origin").unwrap(),
            "https://api.grafbase.com"
        );

        // Test second wildcard domain
        let response = engine
            .raw_execute(
                http::Request::builder()
                    .uri("http://localhost/graphql")
                    .method(http::Method::OPTIONS)
                    .header("Origin", "https://dev.example.com")
                    .header("Access-Control-Request-Method", "POST")
                    .body(Vec::new())
                    .unwrap(),
            )
            .await;

        assert_eq!(response.status(), 200);
        assert_eq!(
            response.headers().get("Access-Control-Allow-Origin").unwrap(),
            "https://dev.example.com"
        );

        let response = engine
            .raw_execute(
                http::Request::builder()
                    .uri("http://localhost/graphql")
                    .method(http::Method::OPTIONS)
                    .header("Origin", "https://malicious.com")
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
fn test_cors_allow_all_origins() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema)
            .with_toml_config(
                r#"
                [cors]
                allow_origins = "*"
            "#,
            )
            .build()
            .await;

        // Test with various origins
        let test_origins = [
            "https://app.grafbase.com",
            "https://example.com",
            "http://localhost:3000",
            "https://malicious.com",
        ];

        for origin in test_origins {
            let response = engine
                .raw_execute(
                    http::Request::builder()
                        .uri("http://localhost/graphql")
                        .method(http::Method::OPTIONS)
                        .header("Origin", origin)
                        .header("Access-Control-Request-Method", "POST")
                        .body(Vec::new())
                        .unwrap(),
                )
                .await;

            assert_eq!(response.status(), 200);
            assert_eq!(response.headers().get("Access-Control-Allow-Origin").unwrap(), "*");
        }
    });
}

#[test]
fn test_cors_allow_all_origins_legacy() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema)
            .with_toml_config(
                r#"
                [cors]
                allow_origins = "any"
            "#,
            )
            .build()
            .await;

        // Test with various origins
        let test_origins = [
            "https://app.grafbase.com",
            "https://example.com",
            "http://localhost:3000",
            "https://malicious.com",
        ];

        for origin in test_origins {
            let response = engine
                .raw_execute(
                    http::Request::builder()
                        .uri("http://localhost/graphql")
                        .method(http::Method::OPTIONS)
                        .header("Origin", origin)
                        .header("Access-Control-Request-Method", "POST")
                        .body(Vec::new())
                        .unwrap(),
                )
                .await;

            assert_eq!(response.status(), 200);
            assert_eq!(response.headers().get("Access-Control-Allow-Origin").unwrap(), "*");
        }
    });
}
