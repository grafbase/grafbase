//! Tests of header forwarding behaviour

use engine_v2::Engine;
use graphql_mocks::{FakeGithubSchema, MockGraphQlServer};
use integration_tests::{federation::EngineV2Ext, runtime};

#[test]
fn test_default_headers() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::builder()
            .with_schema("github", &github_mock)
            .await
            .with_supergraph_config(
                r#"
                    extend schema
                        @allSubgraphs(headers: [
                            {name: "x-foo", value: "BAR"}
                            {name: "x-forwarded", forward: "x-source"}
                        ])
                "#,
            )
            .finish()
            .await;

        engine.execute("query { headers { name value }}").await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "headers": [
          {
            "name": "content-type",
            "value": "application/json"
          },
          {
            "name": "x-foo",
            "value": "BAR"
          },
          {
            "name": "accept",
            "value": "*/*"
          }
        ]
      }
    }
    "###);
}

#[test]
fn test_default_headers_forwarding() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::builder()
            .with_schema("github", &github_mock)
            .await
            .with_supergraph_config(
                r#"
                    extend schema
                        @allSubgraphs(headers: [
                            {name: "x-foo", value: "BAR"}
                            {name: "x-forwarded", forward: "x-source"}
                        ])
                "#,
            )
            .finish()
            .await;

        engine
            .execute("query { headers { name value }}")
            .header("x-source", "boom")
            .await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "headers": [
          {
            "name": "content-type",
            "value": "application/json"
          },
          {
            "name": "x-foo",
            "value": "BAR"
          },
          {
            "name": "x-forwarded",
            "value": "boom"
          },
          {
            "name": "accept",
            "value": "*/*"
          }
        ]
      }
    }
    "###);
}

#[test]
fn test_subgraph_specific_header_forwarding() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::builder()
            .with_schema("github", &github_mock)
            .await
            .with_supergraph_config(
                r#"
                    extend schema
                        @subgraph(name: "other", headers: [
                            {name: "boop", value: "bleep"}
                        ])
                        @subgraph(name: "github", headers: [
                            {name: "x-foo", value: "BAR"}
                            {name: "x-forwarded", forward: "x-source"}
                        ])
                "#,
            )
            .finish()
            .await;

        engine
            .execute("query { headers { name value }}")
            .header("x-source", "boom")
            .await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "headers": [
          {
            "name": "content-type",
            "value": "application/json"
          },
          {
            "name": "x-foo",
            "value": "BAR"
          },
          {
            "name": "x-forwarded",
            "value": "boom"
          },
          {
            "name": "accept",
            "value": "*/*"
          }
        ]
      }
    }
    "###);
}
