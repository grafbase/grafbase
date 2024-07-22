use engine_v2::Engine;
use graphql_mocks::{FakeGithubSchema, MockGraphQlServer, SlowSchema};
use integration_tests::{federation::EngineV2Ext, runtime};

#[test]
fn gateway_timeout() {
    runtime().block_on(async move {
        let slow_subgraph_mock = MockGraphQlServer::new(SlowSchema).await;
        let engine = Engine::builder()
            .with_subgraph("slow", &slow_subgraph_mock)
            .with_timeout(std::time::Duration::from_secs(1))
            .build()
            .await;

        let response = engine
            .execute("query { fast: delay(ms: 0) slow: delay(ms: 500) }")
            .await;

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "fast": 0,
            "slow": 500
          }
        }
        "###);

        let response = engine.execute("query { verySlow: delay(ms: 1500) }").await;

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Gateway timeout",
              "extensions": {
                "code": "GATEWAY_TIMEOUT"
              }
            }
          ]
        }
        "###);
    })
}

#[test]
fn subgraph_timeout() {
    runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;
        let slow_subgraph_mock = MockGraphQlServer::new(SlowSchema).await;
        let engine = Engine::builder()
            .with_subgraph("slow", &slow_subgraph_mock)
            .with_subgraph("github", &github_mock)
            .with_supergraph_config(
                r#"
                extend schema @subgraph(
                    name: "slow",
                    timeout: "1s",
                )
            "#,
            )
            .build()
            .await;

        let response = engine
            .execute("query { serverVersion fast: delay(ms: 0) slow: nullableDelay(ms: 500) }")
            .await;

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "serverVersion": "1",
            "fast": 0,
            "slow": 500
          }
        }
        "###);

        let response = engine
            .execute("query { serverVersion verySlow: delay(ms: 1500) }")
            .await;

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Request to the `slow` subgraph timed out",
              "path": [
                "verySlow"
              ],
              "extensions": {
                "code": "SUBGRAPH_REQUEST_ERROR"
              }
            }
          ]
        }
        "###);

        let response = engine
            .execute("query { serverVersion verySlow: nullableDelay(ms: 1500) }")
            .await;

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "serverVersion": "1",
            "verySlow": null
          },
          "errors": [
            {
              "message": "Request to the `slow` subgraph timed out",
              "path": [
                "verySlow"
              ],
              "extensions": {
                "code": "SUBGRAPH_REQUEST_ERROR"
              }
            }
          ]
        }
        "###);
    })
}
