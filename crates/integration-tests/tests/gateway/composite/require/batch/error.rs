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
                    dummy(id: [ID] @require(field: "[id]")): [JSON] @resolve
                }

                scalar JSON
                "#,
            )
            .with_extension(Resolve::with(|_args| {
                Err(GraphqlError::new("ID 2 doesn't exist!", ErrorCode::BadRequest))
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
                "dummy": null
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
                0,
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
fn invalid_subgraph_response_for_one_element() {
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
                    dummy(id: [ID] @require(field: "[id]")): [Int] @resolve
                }

                "#,
            )
            .with_extension(Resolve::with(|args| {
                Ok(serde_json::Value::Array(args["id"].as_array().unwrap().iter().map(|arg| {
                    if arg == &json!("1") {
                        json!(1)
                    } else {
                        json!("Hi!")
                    }
                }).collect::<Vec<_>>()))
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
fn invalid_subgraph_response_missing_one_element() {
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
                    dummy(id: [ID] @require(field: "[id]")): [String] @resolve
                }

                "#,
            )
            .with_extension(Resolve::with(|args| {
                let first = args["id"].as_array().unwrap().first().unwrap();
                Ok(serde_json::Value::Array(vec![first.clone()]))
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
                "dummy": "1"
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
fn invalid_subgraph_response_missing_two_elements() {
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
                    dummy(id: [ID] @require(field: "[id]")): [String] @resolve
                }

                "#,
            )
            .with_extension(Resolve::with(|_args| {
                Ok(serde_json::Value::Array(vec![]))
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
                "dummy": null
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
                0,
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
fn invalid_subgraph_response_null_response_for_nullable_field() {
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
                    dummy(id: [ID] @require(field: "[id]")): [Int] @resolve
                }

                "#,
            )
            .with_extension(Resolve::with(|_args| {
                Ok(serde_json::Value::Null)
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
                "dummy": null
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
                0,
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
fn invalid_subgraph_response_returns_null() {
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
                    dummy(id: [ID] @require(field: "[id]")): [Int!] @resolve
                }

                "#,
            )
            .with_extension(Resolve::with(|_args| {
                Ok(serde_json::Value::Null)
            }))
            .build()
            .await;

        let response = engine.post("query { products { id dummy } }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "products": [
              null,
              null
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
                0,
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
fn invalid_subgraph_response_returns_not_a_list_nor_null() {
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
                    dummy(id: [ID] @require(field: "[id]")): [Int] @resolve
                }

                "#,
            )
            .with_extension(Resolve::with(|_args| {
                Ok(json!("Hi?"))
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
                "dummy": null
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
                0,
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
                    dummy(id: [ID] @require(field: "[id]")): [Int!] @resolve
                }
                "#,
            )
            .with_extension(Resolve::with(|args| {
                Ok(serde_json::Value::Array(args["id"].as_array().unwrap().iter().map(|arg| {
                    if arg == &json!("1") {
                        json!(1)
                    } else {
                        json!("Hi!")
                    }
                }).collect::<Vec<_>>()))
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
              null
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
