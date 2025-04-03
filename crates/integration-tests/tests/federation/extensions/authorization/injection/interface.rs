use graphql_mocks::dynamic::{DynamicSchema, DynamicSubgraph};
use integration_tests::{
    federation::{AuthorizationExt, Gateway},
    runtime,
};

use crate::federation::extensions::authorization::injection::EchoInjections;

fn subgraph() -> DynamicSubgraph {
    DynamicSchema::builder(
        r#"
        extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

        type Query {
            node: Node
        }

        interface Node @auth(input: {bestfriend: "dog"}, fields: "name") {
            name: String!
            id: ID!
        }

        type User implements Node {
            name: String!
            id: ID!
        }
        "#,
    )
    .with_resolver(
        "Query",
        "node",
        serde_json::json!({"__typename": "User", "id": "789", "name": "Bob"}),
    )
    .into_subgraph("x")
}

#[test]
fn object_field() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(subgraph())
            .with_extension(AuthorizationExt::new(EchoInjections))
            .build()
            .await;

        let response = engine.post("query { node { id } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "node": null
          },
          "errors": [
            {
              "message": "Injection time!",
              "locations": [
                {
                  "line": 1,
                  "column": 9
                }
              ],
              "path": [
                "node"
              ],
              "extensions": {
                "injections": {
                  "query": {
                    "auth": {
                      "Node": {
                        "input": {
                          "bestfriend": "dog"
                        }
                      }
                    }
                  },
                  "response": {
                    "directive_name": "auth",
                    "directive_site": "Node",
                    "items": [
                      {
                        "fields": {
                          "name": "Bob"
                        }
                      }
                    ]
                  }
                },
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "#);
    });
}
