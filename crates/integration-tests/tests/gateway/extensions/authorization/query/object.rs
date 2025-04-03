use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{
    gateway::{AuthorizationExt, Gateway},
    runtime,
};

use crate::gateway::extensions::authorization::{deny_all::DenyAll, user};

#[test]
fn object_type() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

                type Query {
                    user: User
                }

                type User @auth {
                    name: String!
                }
                "#,
                )
                .with_resolver("Query", "user", user())
                .into_subgraph("x"),
            )
            .with_extension(AuthorizationExt::new(DenyAll))
            .build()
            .await;

        let response = engine.post("query { user { name } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "user": null
          },
          "errors": [
            {
              "message": "Not authorized",
              "locations": [
                {
                  "line": 1,
                  "column": 9
                }
              ],
              "path": [
                "user"
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
fn object_within_list() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

                type Query {
                    users: [User]
                }

                type User @auth {
                    name: String!
                }
                "#,
                )
                .with_resolver("Query", "users", serde_json::Value::Array(vec![user(), user()]))
                .into_subgraph("x"),
            )
            .with_extension(AuthorizationExt::new(DenyAll))
            .build()
            .await;

        let response = engine.post("query { users { name } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "users": null
          },
          "errors": [
            {
              "message": "Not authorized",
              "locations": [
                {
                  "line": 1,
                  "column": 9
                }
              ],
              "path": [
                "users"
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
fn object_within_union() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

                type Query {
                    pets: [Pet]
                }

                union Pet = Dog | Cat

                type Dog @auth {
                    name: String!
                }

                type Cat {
                    name: String!
                }
                "#,
                )
                .with_resolver(
                    "Query",
                    "pets",
                    serde_json::json!([{"__typename": "Dog"}, {"__typename": "Cat", "name": "Whiskers"}]),
                )
                .into_subgraph("x"),
            )
            .with_extension(AuthorizationExt::new(DenyAll))
            .build()
            .await;

        let response = engine
            .post("query { pets { ... on Dog { name } ... on Cat { name } } }")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "pets": [
              null,
              {
                "name": "Whiskers"
              }
            ]
          },
          "errors": [
            {
              "message": "Not authorized",
              "locations": [
                {
                  "line": 1,
                  "column": 29
                }
              ],
              "path": [
                "pets",
                0,
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
            "query": "query { pets { __typename ... on Cat { name } } }",
            "operationName": null,
            "variables": {},
            "extensions": {}
          }
        ]
        "#)
    });
}

#[test]
fn explicit_object_behind_interface() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

                type Query {
                    node: Node
                }

                interface Node {
                    name: String!
                }

                type User implements Node @auth {
                    name: String!
                }
                "#,
                )
                .with_resolver("Query", "node", serde_json::json!({"__typename": "User"}))
                .into_subgraph("x"),
            )
            .with_extension(AuthorizationExt::new(DenyAll))
            .build()
            .await;

        let response = engine.post("query { node { ... on User { name } } }").await;
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
                  "column": 30
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
