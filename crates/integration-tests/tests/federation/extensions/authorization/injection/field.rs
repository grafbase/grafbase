use graphql_mocks::dynamic::{DynamicSchema, DynamicSubgraph};
use integration_tests::{
    federation::{AuthorizationExt, Gateway},
    runtime,
};

use crate::federation::extensions::authorization::{injection::EchoInjections, user};

fn subgraph() -> DynamicSubgraph {
    DynamicSchema::builder(
        r#"
        extend schema @link(url: "authorization-1.0.0", import: ["@auth"])
        type Query {
            user: User
            node: Node
        }
        scalar Unit

        interface Node {
            id: ID!
            name: String 
            weight(unit: Unit): Int @auth(input: {bestfriend: "dog"}, args: "*", fields: "name")
        }
        type User implements Node {
            id: ID!
            name: String
            pets(breed: String): [Pet!]! @auth(input: {bestfriend: "dog"}, args: "*", fields: "name")
            weight(unit: Unit): Int
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
        serde_json::json!({"__typename": "User", "name": "Bob", "weight": 30}),
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

        let response = engine
            .post(r#"query { user { pets(breed: "Harrier") { __typename } } }"#)
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "user": null
          },
          "errors": [
            {
              "message": "Injection time!",
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
                "injections": {
                  "query": {
                    "auth": {
                      "User.pets": {
                        "input": {
                          "bestfriend": "dog"
                        },
                        "args": {
                          "breed": "Harrier"
                        }
                      }
                    }
                  },
                  "response": {
                    "directive_name": "auth",
                    "directive_site": "User.pets",
                    "items": [
                      {
                        "fields": {
                          "name": "Peter"
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

#[test]
fn interface_field() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(subgraph())
            .with_extension(AuthorizationExt::new(EchoInjections))
            .build()
            .await;

        let response = engine.post(r#"query { node { weight(unit: "kg") } }"#).await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "node": {
              "weight": null
            }
          },
          "errors": [
            {
              "message": "Injection time!",
              "locations": [
                {
                  "line": 1,
                  "column": 16
                }
              ],
              "path": [
                "node",
                "weight"
              ],
              "extensions": {
                "injections": {
                  "query": {
                    "auth": {
                      "Node.weight": {
                        "input": {
                          "bestfriend": "dog"
                        },
                        "args": {
                          "unit": "kg"
                        }
                      }
                    }
                  },
                  "response": {
                    "directive_name": "auth",
                    "directive_site": "Node.weight",
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
