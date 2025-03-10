use engine::Engine;
use graphql_mocks::dynamic::{DynamicSchema, DynamicSubgraph};
use integration_tests::{federation::EngineExt, runtime};

use crate::federation::extensions::authorization::{AuthorizationExt, deny_some::DenySites, grant_all::GrantAll, user};

fn subgraph() -> DynamicSubgraph {
    DynamicSchema::builder(
        r#"
        extend schema @link(url: "authorization-1.0.0", import: ["@auth"])
        type Query {
            user: User
            node: Node
        }
        interface Node {
            id: ID!
            name: String @auth(fields: "id")
        }
        type User implements Node {
            id: ID!
            name: String
            pets: [Pet!]! @auth(fields: "id")
        }
        union Pet = Dog | Cat
        type Dog {
            id: ID!
            name: String!
        }
        type Cat {
            id: ID!
        }
        "#,
    )
    .with_resolver("Query", "user", user())
    .with_resolver(
        "Query",
        "node",
        serde_json::json!({"__typename": "User", "id": "890", "name": "Bob"}),
    )
    .into_subgraph("x")
}

#[test]
fn grant_object_field() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(subgraph())
            .with_extension(AuthorizationExt::new(GrantAll))
            .build()
            .await;

        let response = engine.post("query { user { pets { __typename } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "user": {
              "pets": [
                {
                  "__typename": "Dog"
                },
                {
                  "__typename": "Cat"
                }
              ]
            }
          }
        }
        "#);
    });
}

#[test]
fn deny_object_field_at_query_stage() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(subgraph())
            .with_extension(AuthorizationExt::new(DenySites::query(vec!["User.pets"])))
            .build()
            .await;

        let response = engine.post("query { user { pets { __typename } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "user": null
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
                "user",
                "pets"
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
            "query": "query { user { __typename @skip(if: true) } }",
            "operationName": null,
            "variables": {},
            "extensions": {}
          }
        ]
        "#)
    });
}

#[test]
fn deny_object_field_at_response_stage() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(subgraph())
            .with_extension(AuthorizationExt::new(DenySites::response(vec!["User.pets"])))
            .build()
            .await;

        let response = engine.post("query { user { pets { __typename } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "user": null
          },
          "errors": [
            {
              "message": "Unauthorized at response stage",
              "locations": [
                {
                  "line": 1,
                  "column": 16
                }
              ],
              "path": [
                "user",
                "pets"
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

#[test]
fn grant_interface_field() {
    runtime().block_on(async move {
        let engine = Engine::builder()
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
fn deny_interface_field_at_query_stage() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(subgraph())
            .with_extension(AuthorizationExt::new(DenySites::query(vec!["Node.name"])))
            .build()
            .await;

        let response = engine.post("query { node { name } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "node": {
              "name": null
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
            "query": "query { node { __typename @skip(if: true) } }",
            "operationName": null,
            "variables": {},
            "extensions": {}
          }
        ]
        "#)
    });
}

#[test]
fn deny_interface_field_at_response_stage() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(subgraph())
            .with_extension(AuthorizationExt::new(DenySites::response(vec!["Node.name"])))
            .build()
            .await;

        let response = engine.post("query { node { name } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "node": {
              "name": null
            }
          },
          "errors": [
            {
              "message": "Unauthorized at response stage",
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
