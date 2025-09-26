use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{
    gateway::{AuthorizationExt, Gateway},
    runtime,
};

use crate::gateway::extensions::authorization::{deny_all::DenyAll, user};

#[test]
fn required_item_required_parent() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema @link(url: "authorization", import: ["@auth"])

                type Query {
                    user: User
                }

                type User {
                    pets: [Pet!]!
                }

                union Pet = Dog | Cat

                type Dog {
                    id: ID!
                    name: String! @auth
                }

                type Cat {
                    id: ID!
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

        let response = engine
            .post("query { user { pets { __typename ... on Dog { id } } } }")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "user": {
              "pets": [
                {
                  "__typename": "Dog",
                  "id": "1"
                },
                {
                  "__typename": "Cat"
                }
              ]
            }
          }
        }
        "#);

        let response = engine
            .post("query { user { pets { __typename ... on Dog { id name } } } }")
            .await;
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
                  "column": 50
                }
              ],
              "path": [
                "user",
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
            "query": "query { user { pets { __typename ... on Dog { id } } } }",
            "operationName": null,
            "variables": {},
            "extensions": {}
          },
          {
            "query": "query { user { pets { __typename ... on Dog { id } } } }",
            "operationName": null,
            "variables": {},
            "extensions": {}
          }
        ]
        "#)
    });
}

#[test]
fn required_item_nullable_parent() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema @link(url: "authorization", import: ["@auth"])

                type Query {
                    user: User
                }

                type User {
                    pets: [Pet!]
                }

                union Pet = Dog | Cat

                type Dog {
                    id: ID!
                    name: String! @auth
                }

                type Cat {
                    id: ID!
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

        let response = engine
            .post("query { user { pets { __typename ... on Dog { id } } } }")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "user": {
              "pets": [
                {
                  "__typename": "Dog",
                  "id": "1"
                },
                {
                  "__typename": "Cat"
                }
              ]
            }
          }
        }
        "#);

        let response = engine
            .post("query { user { pets { __typename ... on Dog { id name } } } }")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "user": {
              "pets": null
            }
          },
          "errors": [
            {
              "message": "Not authorized",
              "locations": [
                {
                  "line": 1,
                  "column": 50
                }
              ],
              "path": [
                "user",
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
            "query": "query { user { pets { __typename ... on Dog { id } } } }",
            "operationName": null,
            "variables": {},
            "extensions": {}
          },
          {
            "query": "query { user { pets { __typename ... on Dog { id } } } }",
            "operationName": null,
            "variables": {},
            "extensions": {}
          }
        ]
        "#)
    });
}

#[test]
fn nullable_item() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema @link(url: "authorization", import: ["@auth"])

                type Query {
                    user: User
                }

                type User {
                    pets: [Pet]!
                }

                union Pet = Dog | Cat

                type Dog {
                    id: ID!
                    name: String! @auth
                }

                type Cat {
                    id: ID!
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

        let response = engine
            .post("query { user { pets { __typename ... on Dog { id } } } }")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "user": {
              "pets": [
                {
                  "__typename": "Dog",
                  "id": "1"
                },
                {
                  "__typename": "Cat"
                }
              ]
            }
          }
        }
        "#);

        let response = engine
            .post("query { user { pets { __typename ... on Dog { id name } } } }")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "user": {
              "pets": [
                null,
                {
                  "__typename": "Cat"
                }
              ]
            }
          },
          "errors": [
            {
              "message": "Not authorized",
              "locations": [
                {
                  "line": 1,
                  "column": 50
                }
              ],
              "path": [
                "user",
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
            "query": "query { user { pets { __typename ... on Dog { id } } } }",
            "operationName": null,
            "variables": {},
            "extensions": {}
          },
          {
            "query": "query { user { pets { __typename ... on Dog { id } } } }",
            "operationName": null,
            "variables": {},
            "extensions": {}
          }
        ]
        "#)
    });
}
