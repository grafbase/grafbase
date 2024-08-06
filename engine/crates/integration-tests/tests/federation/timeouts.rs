use engine_v2::Engine;
use graphql_mocks::{FakeGithubSchema, SlowSchema};
use integration_tests::{federation::EngineV2Ext, runtime};

#[test]
fn gateway_timeout() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(SlowSchema)
            .with_toml_config(
                r###"
                [gateway]
                timeout = "1s"
                "###,
            )
            .build()
            .await;

        let response = engine.post("query { fast: delay(ms: 0) slow: delay(ms: 500) }").await;

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "fast": 0,
            "slow": 500
          }
        }
        "###);

        let response = engine.post("query { verySlow: delay(ms: 1500) }").await;

        insta::assert_json_snapshot!(response, @r###"
        {
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
        let engine = Engine::builder()
            .with_subgraph(SlowSchema)
            .with_subgraph(FakeGithubSchema)
            .with_sdl_config(
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
            .post("query { serverVersion fast: delay(ms: 0) slow: nullableDelay(ms: 500) }")
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

        let response = engine.post("query { serverVersion verySlow: delay(ms: 1500) }").await;

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Request to subgraph 'slow' failed with: Request timeout",
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
            .post("query { serverVersion verySlow: nullableDelay(ms: 1500) }")
            .await;

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "serverVersion": "1",
            "verySlow": null
          },
          "errors": [
            {
              "message": "Request to subgraph 'slow' failed with: Request timeout",
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
