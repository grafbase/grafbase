use engine_v2::Engine;
use graphql_mocks::{MockGraphQlServer, SlowSchema};
use integration_tests::{federation::EngineV2Ext, runtime};

#[test]
fn gateway_timeout() {
    runtime().block_on(async move {
        let slow_subgraph_mock = MockGraphQlServer::new(SlowSchema).await;
        let engine = Engine::builder()
            .with_subgraph("slow", &slow_subgraph_mock)
            .with_timeout(std::time::Duration::from_secs(3))
            .build()
            .await;

        let response = engine.execute("query { fastField oneSecondField }").await;

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "fastField": 100,
            "oneSecondField": 200
          }
        }
        "###);

        let response = engine
            .execute("query { fastField oneSecondField fiveSecondField }")
            .await;

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
        let slow_subgraph_mock = MockGraphQlServer::new(SlowSchema).await;
        let engine = Engine::builder()
            .with_subgraph("slow", &slow_subgraph_mock)
            .with_supergraph_config(
                r#"
                extend schema @subgraph(
                    name: "slow",
                    timeout: "3s",
                )
            "#,
            )
            .build()
            .await;

        let response = engine.execute("query { fastField oneSecondField }").await;

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "fastField": 100,
            "oneSecondField": 200
          }
        }
        "###);

        let response = engine
            .execute("query { fastField oneSecondField fiveSecondField }")
            .await;

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Request to the `slow` subgraph timed out",
              "path": [
                "fastField"
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
