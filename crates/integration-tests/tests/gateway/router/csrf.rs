use graphql_mocks::FakeGithubSchema;
use integration_tests::{gateway::Gateway, runtime};

#[test]
fn no_header() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema)
            .with_toml_config(
                r#"
                [csrf]
                enabled = true
            "#,
            )
            .build()
            .await;

        let response = engine.post("{ __typename }").await;
        assert_eq!(response.status, http::StatusCode::FORBIDDEN);
    })
}

#[test]
fn with_header() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema)
            .with_toml_config(
                r#"
                [csrf]
                enabled = true
            "#,
            )
            .build()
            .await;

        let response = engine
            .post("{ __typename }")
            .header("x-grafbase-csrf-protection", "1")
            .await;

        insta::assert_json_snapshot!(&response, @r#"
        {
          "data": {
            "__typename": "Query"
          }
        }
        "#);
        assert_eq!(response.status, 200);
    })
}

#[test]
fn with_custom_header() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema)
            .with_toml_config(
                r#"
                [csrf]
                enabled = true
                header_name = "my-custom-csrf"
            "#,
            )
            .build()
            .await;

        let response = engine.post("{ __typename }").await;
        assert_eq!(response.status, http::StatusCode::FORBIDDEN);

        let response = engine.post("{ __typename }").header("my-custom-csrf", "1").await;
        insta::assert_json_snapshot!(&response, @r#"
        {
          "data": {
            "__typename": "Query"
          }
        }
        "#);
        assert_eq!(response.status, 200);
    })
}

#[test]
fn mcp() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema)
            .with_toml_config(
                r#"
                [csrf]
                enabled = true

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
                    .method(http::Method::POST)
                    .body(Vec::new())
                    .unwrap(),
            )
            .await;
        assert_eq!(response.status(), http::StatusCode::FORBIDDEN);
    })
}
