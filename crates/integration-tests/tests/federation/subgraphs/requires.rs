use engine::Engine;
use graphql_mocks::dynamic::{DynamicSchema, EntityResolverContext};
use integration_tests::{
    federation::{EngineExt, GraphqlResponse},
    runtime,
};
use serde_json::json;

#[test]
fn simple_requires() {
    let response = runtime().block_on(super::execute(
        r"
        query ExampleQuery {
            topProducts {
                name
                reviews {
                    author {
                        username
                        trustworthiness
                    }
                }
            }
        }
        ",
    ));

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "topProducts": [
          {
            "name": "Trilby",
            "reviews": [
              {
                "author": {
                  "username": "Me",
                  "trustworthiness": "REALLY_TRUSTED"
                }
              }
            ]
          },
          {
            "name": "Fedora",
            "reviews": [
              {
                "author": {
                  "username": "Me",
                  "trustworthiness": "REALLY_TRUSTED"
                }
              }
            ]
          },
          {
            "name": "Boater",
            "reviews": [
              {
                "author": {
                  "username": "User 7777",
                  "trustworthiness": "KINDA_TRUSTED"
                }
              }
            ]
          },
          {
            "name": "Jeans",
            "reviews": []
          },
          {
            "name": "Pink Jeans",
            "reviews": [
              {
                "author": null
              }
            ]
          }
        ]
      }
    }
    "###);
}

#[test]
fn requires_with_arguments() {
    let response = runtime().block_on(super::execute(
        r"
        query ExampleQuery {
            topProducts {
                name
                weight(unit: GRAM)
                shippingEstimate
            }
        }
        ",
    ));

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "topProducts": [
          {
            "name": "Trilby",
            "weight": 100.0,
            "shippingEstimate": 1
          },
          {
            "name": "Fedora",
            "weight": 200.0,
            "shippingEstimate": 1
          },
          {
            "name": "Boater",
            "weight": 300.0,
            "shippingEstimate": 1
          },
          {
            "name": "Jeans",
            "weight": 400.0,
            "shippingEstimate": 3
          },
          {
            "name": "Pink Jeans",
            "weight": 500.0,
            "shippingEstimate": 3
          }
        ]
      }
    }
    "###);
}

#[test]
fn requires_with_fragments() {
    async fn run_with_user(user: serde_json::Value) -> GraphqlResponse {
        let gateway = Engine::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                type Query {
                    user: User!
                }

                type User @key(fields: "id") {
                    id: ID!
                    node: Node
                }

                interface Node {
                    id: ID!
                }

                type A implements Node {
                    id: ID!
                }

                type B implements Node {
                    id: ID!
                }
                "#,
                )
                .with_resolver("Query", "user", user)
                .into_subgraph("x"),
            )
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                scalar Any

                type User @key(fields: "id") {
                    id: ID!
                    node: Node @external
                    repr: Any @requires(fields: "node { ... on B { id } }")
                }

                interface Node {
                    id: ID!
                }

                type B {
                    id: ID! @external
                }
                "#,
                )
                .with_entity_resolver("User", |ctx: EntityResolverContext<'_>| {
                    let mut repr = ctx.representation.clone();
                    repr.remove("__typename");

                    Some(json!({ "__typename": "User", "repr": repr }))
                })
                .into_subgraph("y"),
            )
            .build()
            .await;

        gateway.post("{ user { repr } }").await
    }

    runtime().block_on(async {
        let response = run_with_user(serde_json::json!({
            "id": "1",
            "node": {"__typename": "B", "id": "b"}
        }))
        .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "user": {
              "repr": {
                "node": {
                  "id": "b"
                },
                "id": "1"
              }
            }
          }
        }
        "#);

        let response = run_with_user(serde_json::json!({
            "id": "1",
            "node": {"__typename": "A", "id": "a"}
        }))
        .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "user": {
              "repr": {
                "node": {},
                "id": "1"
              }
            }
          }
        }
        "#);
    })
}
