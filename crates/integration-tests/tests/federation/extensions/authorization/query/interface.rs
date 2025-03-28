use engine::Engine;
use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{
    federation::{AuthorizationExt, EngineExt},
    runtime,
};

use crate::federation::extensions::authorization::deny_some::DenySites;

#[test]
fn interface_fields() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

                type Query {
                    node: Node
                }

                interface Node {
                    name: String @auth
                    id: ID @auth
                }

                type User implements Node {
                    name: String
                    id: ID
                }
                "#,
                )
                .with_resolver("Query", "node", serde_json::json!({"__typename": "User", "id": "980"}))
                .into_subgraph("x"),
            )
            .with_extension(AuthorizationExt::new(DenySites::query(vec!["Node.name"])))
            .build()
            .await;

        let response = engine.post("query { node { name id } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "node": {
              "name": null,
              "id": "980"
            }
          },
          "errors": [
            {
              "message": "Unauthorized at query stage",
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

        let sent = engine.drain_graphql_requests_sent_to_by_name("x");
        insta::assert_json_snapshot!(sent, @r#"
        [
          {
            "query": "query { node { id } }",
            "operationName": null,
            "variables": {},
            "extensions": {}
          }
        ]
        "#)
    });
}

#[test]
fn interface_type() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

                type Query {
                    node: Node
                }

                interface Node @auth {
                    name: String!
                    id: ID!
                }

                type User implements Node {
                    name: String!
                    id: ID!
                }
                "#,
                )
                .into_subgraph("x"),
            )
            .with_extension(AuthorizationExt::new(DenySites::query(vec!["Node"])))
            .build()
            .await;

        let response = engine.post("query { node { name id } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "node": null
          },
          "errors": [
            {
              "message": "Unauthorized at query stage",
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
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "#);

        let sent = engine.drain_graphql_requests_sent_to_by_name("x");
        insta::assert_json_snapshot!(sent, @r#"
        [
          {
            "query": "query { __typename @skip(if: true) }",
            "operationName": null,
            "variables": {},
            "extensions": {}
          }
        ]
        "#)
    });
}
