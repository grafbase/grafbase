use engine::{ErrorCode, GraphqlError};
use integration_tests::{gateway::Gateway, runtime};
use serde_json::json;

use super::{Resolve, gql_product};

#[test]
fn resolver_error() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_product())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "resolver", import: ["@resolve"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@require", "@key", "@external"])

                type Product @key(fields: "id") {
                    id: ID!
                    dummy(id: ID @require(field: "id")): JSON @resolve
                }

                scalar JSON
                "#,
            )
            .with_extension(Resolve::with(|args| {
                if args["id"] == json!("1") {
                    Ok(args)
                } else {
                    Err(GraphqlError::new("ID 2 doesn't exist!", ErrorCode::BadRequest))
                }
            }))
            .build()
            .await;

        let response = engine.post("query { products { id dummy } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "id": "1",
                "dummy": {
                  "id": "1"
                }
              },
              {
                "id": "2",
                "dummy": null
              }
            ]
          },
          "errors": [
            {
              "message": "ID 2 doesn't exist!",
              "locations": [
                {
                  "line": 1,
                  "column": 23
                }
              ],
              "path": [
                "products",
                1,
                "dummy"
              ],
              "extensions": {
                "code": "BAD_REQUEST"
              }
            }
          ]
        }
        "#);
    })
}

#[test]
fn invalid_subgraph_response() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_product())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "resolver", import: ["@resolve"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@require", "@key", "@external"])

                type Product @key(fields: "id") {
                    id: ID!
                    dummy(id: ID @require(field: "id")): Int @resolve
                }

                "#,
            )
            .with_extension(Resolve::with(|args| {
                if args["id"] == json!("1") {
                    Ok(json!(1))
                } else {
                    Ok(json!("Hi!"))
                }
            }))
            .build()
            .await;

        let response = engine.post("query { products { id dummy } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "id": "1",
                "dummy": 1
              },
              {
                "id": "2",
                "dummy": null
              }
            ]
          },
          "errors": [
            {
              "message": "Invalid response from subgraph",
              "locations": [
                {
                  "line": 1,
                  "column": 23
                }
              ],
              "path": [
                "products",
                1,
                "dummy"
              ],
              "extensions": {
                "code": "SUBGRAPH_INVALID_RESPONSE_ERROR"
              }
            }
          ]
        }
        "#);
    })
}

#[test]
fn null_propagation() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph(gql_product())
            .with_subgraph_sdl(
                "ext",
                r#"
                extend schema
                    @link(url: "resolver", import: ["@resolve"])
                    @link(url: "https://specs.grafbase.com/composite-schemas/v1", import: ["@require", "@key", "@external"])

                type Product @key(fields: "id") {
                    id: ID!
                    dummy(id: ID @require(field: "id")): JSON! @resolve
                }

                scalar JSON
                "#,
            )
            .with_extension(Resolve::with(|args| {
                if args["id"] == json!("1") {
                    Ok(args)
                } else {
                    Err(GraphqlError::new("ID 2 doesn't exist!", ErrorCode::BadRequest))
                }
            }))
            .build()
            .await;

        let response = engine.post("query { products { id dummy } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              {
                "id": "1",
                "dummy": {
                  "id": "1"
                }
              },
              null
            ]
          },
          "errors": [
            {
              "message": "ID 2 doesn't exist!",
              "locations": [
                {
                  "line": 1,
                  "column": 23
                }
              ],
              "path": [
                "products",
                1,
                "dummy"
              ],
              "extensions": {
                "code": "BAD_REQUEST"
              }
            }
          ]
        }
        "#);
    })
}
