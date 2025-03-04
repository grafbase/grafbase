use engine::Engine;
use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{federation::EngineExt, runtime};

use crate::federation::extensions::authorization::{SimpleAuthExt, deny_all::DenyAll, user};

#[test]
fn required_field_required_parent() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema @link(url: "simple-auth-1.0.0", import: ["@auth"])

                type Query {
                    user: User
                }

                type User {
                    address: Address!
                }

                type Address {
                    city: String! @auth
                    country: String!
                }
                "#,
                )
                .with_resolver("Query", "user", user())
                .into_subgraph("x"),
            )
            .with_extension(SimpleAuthExt::new(DenyAll))
            .build()
            .await;

        let response = engine.post("query { user { address { country } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "user": {
              "address": {
                "country": "USA"
              }
            }
          }
        }
        "#);

        let response = engine.post("query { user { address { country city } } }").await;
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
                  "column": 34
                }
              ],
              "path": [
                "user",
                "address",
                "city"
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
fn required_field_nullable_parent() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema @link(url: "simple-auth-1.0.0", import: ["@auth"])

                type Query {
                    user: User
                }

                type User {
                    address: Address
                }

                type Address {
                    city: String! @auth
                    country: String!
                }
                "#,
                )
                .with_resolver("Query", "user", user())
                .into_subgraph("x"),
            )
            .with_extension(SimpleAuthExt::new(DenyAll))
            .build()
            .await;

        let response = engine.post("query { user { address { country } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "user": {
              "address": {
                "country": "USA"
              }
            }
          }
        }
        "#);

        let response = engine.post("query { user { address { country city } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "user": {
              "address": null
            }
          },
          "errors": [
            {
              "message": "Not authorized",
              "locations": [
                {
                  "line": 1,
                  "column": 34
                }
              ],
              "path": [
                "user",
                "address",
                "city"
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
fn nullable_field() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema @link(url: "simple-auth-1.0.0", import: ["@auth"])

                type Query {
                    user: User
                }

                type User {
                    address: Address
                }

                type Address {
                    city: String @auth
                    country: String!
                }
                "#,
                )
                .with_resolver("Query", "user", user())
                .into_subgraph("x"),
            )
            .with_extension(SimpleAuthExt::new(DenyAll))
            .build()
            .await;

        let response = engine.post("query { user { address { country } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "user": {
              "address": {
                "country": "USA"
              }
            }
          }
        }
        "#);

        let response = engine.post("query { user { address { country city } } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "user": {
              "address": {
                "country": "USA",
                "city": null
              }
            }
          },
          "errors": [
            {
              "message": "Not authorized",
              "locations": [
                {
                  "line": 1,
                  "column": 34
                }
              ],
              "path": [
                "user",
                "address",
                "city"
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
