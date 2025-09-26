use graphql_mocks::FakeGithubSchema;
use integration_tests::{gateway::Gateway, runtime};

#[test]
fn top_level_typename() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema::default())
            .with_toml_config(
                r#"
            [graph.error_code_mapping]
            OPERATION_VALIDATION_ERROR = "CUSTOM_VALIDATION_ERROR"
            "#,
            )
            .build()
            .await;

        engine.post("query { unknown }").await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "errors": [
        {
          "message": "Query does not have a field named 'unknown'.",
          "locations": [
            {
              "line": 1,
              "column": 9
            }
          ],
          "extensions": {
            "code": "CUSTOM_VALIDATION_ERROR"
          }
        }
      ]
    }
    "#);
}
