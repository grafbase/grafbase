use engine::Engine;
use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{federation::EngineExt, runtime};

use crate::federation::extensions::authorization::{AuthorizationExt, injection::EchoInjections, user};

#[test]
fn complex_field_set() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

                type Query {
                    user: User
                }

                type User @auth(input: { bestfriend: "dog" }, fields: "id name address { street } friends { name } pets { ... on Dog { name } }") {
                    id: ID!
                    name: String!
                    age: Int!
                    address: Address
                    friends: [User!]
                    pets: [Pet!]!
                }

                type Address {
                    street: String!
                    city: String!
                    country: String!
                }

                union Pet = Dog | Cat

                type Dog {
                    id: ID!
                    name: String!
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
            .with_extension(AuthorizationExt::new(EchoInjections))
            .build()
            .await;

        let response = engine.post("query { user { id } }").await;
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
                  "column": 9
                }
              ],
              "path": [
                "user"
              ],
              "extensions": {
                "injections": {
                  "query": {
                    "auth": {
                      "User": {
                        "input": {
                          "bestfriend": "dog"
                        }
                      }
                    }
                  },
                  "response": {
                    "directive_name": "auth",
                    "directive_site": "User",
                    "items": [
                      {
                        "id": "1",
                        "name": "Peter",
                        "address": {
                          "street": "123 Main St"
                        },
                        "friends": [
                          {
                            "name": "Alice"
                          },
                          {
                            "name": "Bob"
                          }
                        ],
                        "pets": [
                          {
                            "name": "Fido"
                          },
                          {}
                        ]
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
