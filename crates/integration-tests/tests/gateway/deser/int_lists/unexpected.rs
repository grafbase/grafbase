use graphql_mocks::dynamic::{DynamicSchema, EntityResolverContext};
use integration_tests::gateway::{Gateway, GraphqlResponse};
use serde_json::json;

use crate::gateway::extensions::resolver::ResolverExt;

fn run(ints: serde_json::Value, query: &str) -> GraphqlResponse {
    integration_tests::runtime().block_on(async {
        Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "resolver-1.0.0", import: ["@resolve"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@key"])

                type Query {
                    me: User @resolve
                }

                type User @key(fields: "id") {
                    id: ID!
                    ints: [Int!]!
                }
                "#,
            )
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                    type Query {
                        dummy: String
                    }

                    type User @key(fields: "id") {
                        id: ID!
                        ints: [Int!] @external
                        repr: JSON @requires(fields: "ints")
                    }

                    scalar JSON
                    "#,
                )
                .with_entity_resolver("User", |ctx: EntityResolverContext<'_>| {
                    let mut repr = ctx.representation.clone();
                    repr.remove("__typename");

                    Some(json!({ "repr": repr }))
                })
                .into_subgraph("b"),
            )
            .with_extension(ResolverExt::json(json!({
                "id": "1",
                "ints": ints,
            })))
            .build()
            .await
            .post(query)
            .await
    })
}

#[test]
fn valid_ints() {
    let response = run(
        json!([1, 2, 3]),
        r#"
        query {
            me {
                repr
            }
        }
        "#,
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "me": {
          "repr": {
            "id": "1",
            "ints": [
              1,
              2,
              3
            ]
          }
        }
      }
    }
    "#);
}

#[test]
fn invalid_ints_should_prevent_further_subgraph_request() {
    let response = run(
        json!([1, null, 3]),
        r#"
        query {
            me {
                repr
            }
        }
        "#,
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "me": {
          "repr": null
        }
      }
    }
    "#);
}

#[test]
fn extra_invalid_ints_should_prevent_further_subgraph_request() {
    let response = run(
        json!([1, null, 3]),
        r#"
        query {
            me {
                ints
                repr
            }
        }
        "#,
    );
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "me": {
          "ints": null,
          "repr": null
        }
      },
      "errors": [
        {
          "message": "Invalid response from subgraph",
          "locations": [
            {
              "line": 4,
              "column": 17
            }
          ],
          "path": [
            "me",
            "ints",
            1
          ],
          "extensions": {
            "code": "SUBGRAPH_INVALID_RESPONSE_ERROR"
          }
        }
      ]
    }
    "#);
}
