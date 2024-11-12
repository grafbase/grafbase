use engine_v2::Engine;
use graphql_mocks::{AlmostEmptySchema, FakeGithubSchema};
use integration_tests::{federation::EngineV2Ext, runtime};
use serde_json::json;

#[test]
fn supports_custom_scalars() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(FakeGithubSchema).build().await;

        engine.post("query { favoriteRepository }").await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "favoriteRepository": {
          "owner": "rust-lang",
          "name": "rust"
        }
      }
    }
    "###);
}

#[test]
fn supports_unused_builtin_scalars() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(AlmostEmptySchema).build().await;

        engine
            .post("query Blah($id: ID!) { string(input: $id) }")
            .variables(json!({"id": "1"}))
            .await
    });

    // Bit of a poor test this because we can never pass a valid query that makes use of a scalar that doesn't exist.
    // But so long as any errors below don't include "Unknown type `ID` or similar I think we're good"

    insta::assert_json_snapshot!(response, @r###"
    {
      "errors": [
        {
          "message": "Variable $id doesn't have the right type. Declared as 'ID!' but used as 'String!'",
          "locations": [
            {
              "line": 1,
              "column": 38
            }
          ],
          "extensions": {
            "code": "OPERATION_VALIDATION_ERROR"
          }
        }
      ]
    }
    "###);
}
