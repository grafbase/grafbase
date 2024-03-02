use engine_v2::Engine;
use graphql_mocks::{MockGraphQlServer, StateMutationSchema};
use integration_tests::{federation::EngineV2Ext, runtime};

#[test]
fn mutations_should_be_executed_sequentially() {
    runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(StateMutationSchema::default()).await;
        let engine = Engine::builder()
            .with_schema("schema", &github_mock)
            .await
            .finish()
            .await;

        // sanity check
        let response = engine.execute("query { value }").await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "value": 0
          }
        }
        "###);

        let response = engine
            .execute(
                r"
                mutation {
                    first: set(val: 1)
                    second: multiply(by: 2)
                    third: multiply(by: 7)
                    fourth: set(val: 3)
                    fifth: multiply(by: 11)
                }
                ",
            )
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "first": 1,
            "second": 2,
            "third": 14,
            "fourth": 3,
            "fifth": 33
          }
        }
        "###);

        let response = engine.execute("query { value }").await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "value": 33
          }
        }
        "###);
    });
}

#[test]
fn mutation_failure_should_stop_later_executions_if_required() {
    runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(StateMutationSchema::default()).await;
        let engine = Engine::builder()
            .with_schema("schema", &github_mock)
            .await
            .finish()
            .await;

        // sanity check
        let response = engine.execute("query { value }").await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "value": 0
          }
        }
        "###);

        let response = engine
            .execute(
                r"
                mutation {
                    first: set(val: 1)
                    second: multiply(by: 2)
                    faillible
                    third: multiply(by: 7)
                }
                ",
            )
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "first": 1,
            "second": 2,
            "faillible": null,
            "third": 14
          },
          "errors": [
            {
              "message": "Upstream error: This mutation always fails",
              "path": [
                "faillible"
              ],
              "extensions": {
                "upstream_locations": [
                  {
                    "line": 2,
                    "column": 2
                  }
                ],
                "upstream_path": [
                  "faillible"
                ]
              }
            }
          ]
        }
        "###);

        let response = engine.execute("query { value }").await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "value": 14
          }
        }
        "###);

        let response = engine
            .execute(
                r"
                mutation {
                    first: set(val: 1)
                    second: multiply(by: 2)
                    fail
                    third: multiply(by: 7)
                }
                ",
            )
            .await;

        // the error isn't great, we could definitely do better. At least it's somewhat clear.
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Upstream error: This mutation always fails",
              "path": [
                "fail"
              ],
              "extensions": {
                "upstream_locations": [
                  {
                    "line": 2,
                    "column": 2
                  }
                ],
                "upstream_path": [
                  "fail"
                ]
              }
            }
          ]
        }
        "###);

        let response = engine.execute("query { value }").await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "value": 2
          }
        }
        "###);
    });
}
