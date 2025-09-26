use graphql_mocks::dynamic::{DynamicSchema, DynamicSubgraph};
use integration_tests::{
    gateway::{AuthorizationExt, Gateway},
    runtime,
};

use crate::gateway::extensions::authorization::{deny_some::DenySites, grant_all::GrantAll};

fn subgraph() -> DynamicSubgraph {
    DynamicSchema::builder(
        r#"
        extend schema @link(url: "authorization", import: ["@auth"])

        type Query {
            node: Node
        }

        interface Node @auth(fields: "id") {
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
        serde_json::json!({"__typename": "User", "id": "90", "name": "Bob"}),
    )
    .into_subgraph("x")
}

#[test]
fn grant_interface_type() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(subgraph())
            .with_extension(AuthorizationExt::new(GrantAll))
            .build()
            .await;

        let response = engine.post("query { node { name } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "node": {
              "name": "Bob"
            }
          }
        }
        "#);
    });
}

#[test]
fn deny_interface_type_at_query_stage() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(subgraph())
            .with_extension(AuthorizationExt::new(DenySites::query(["Node"])))
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

#[test]
fn deny_interface_type_at_response_stage() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(subgraph())
            .with_extension(AuthorizationExt::new(DenySites::response(["Node"])))
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
              "message": "Unauthorized at response stage",
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
    });
}
