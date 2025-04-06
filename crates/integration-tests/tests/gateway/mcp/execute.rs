use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{gateway::Gateway, runtime};
use serde_json::json;

#[test]
fn execute_valid_graphql_query() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                type Query {
                    user: User
                }

                type User {
                    id: ID!
                    name: String!
                }
                "#,
                )
                .with_resolver("Query", "user", json!({"id": "1", "name": "Alice"}))
                .into_subgraph("y"),
            )
            .with_toml_config(
                r#"
                [mcp]
                enabled = true
            "#,
            )
            .build()
            .await;

        let mut stream = engine.mcp("/mcp").await;

        let response = stream
            .call_tool("execute", json!({"query": "query { user { name } }"}))
            .await;

        insta::assert_json_snapshot!(&response, @r#"
        {
          "result": {
            "content": [
              {
                "data": {
                  "user": {
                    "name": "Alice"
                  }
                }
              }
            ],
            "is_error": null
          }
        }
        "#);
    });
}
