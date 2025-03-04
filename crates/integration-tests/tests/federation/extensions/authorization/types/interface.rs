use engine::Engine;
use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{federation::EngineExt, runtime};

use crate::federation::extensions::authorization::{SimpleAuthExt, deny_all::DenyAll};

#[test]
fn explicit_object_behind_interface() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema @link(url: "simple-auth-1.0.0", import: ["@auth"])

                type Query {
                    node: Node
                }

                interface Node {
                    name: String! @auth
                }

                type User implements Node {
                    name: String!
                }
                "#,
                )
                .with_resolver(
                    "Query",
                    "node",
                    serde_json::json!({"__typename": "User", "name": "Alice"}),
                )
                .into_subgraph("x"),
            )
            .with_extension(SimpleAuthExt::new(DenyAll))
            .build()
            .await;

        let response = engine.post("query { node { name } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "node": null
          },
          "errors": [
            {
              "message": "Not authorized",
              "locations": [
                {
                  "line": 1,
                  "column": 16
                }
              ],
              "path": [
                "node",
                "name"
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "#);
    });
}
