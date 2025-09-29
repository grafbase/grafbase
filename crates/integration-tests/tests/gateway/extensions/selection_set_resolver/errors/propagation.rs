use integration_tests::{gateway::Gateway, runtime};
use serde_json::json;

use crate::gateway::extensions::selection_set_resolver::StaticSelectionSetResolverExt;

const SDL: &str = r#"
extend schema
    @link(url: "static", import: ["@init"])
    @init

type Nullable {
    id: ID
    int: Int
    str: String
    float: Float
    bool: Boolean
    nullable: Nullable
    nullableList: [Nullable]
    required: Required!
    requiredList: [Required!]
}

type Required {
    id: ID!
    int: Int!
    str: String!
    float: Float!
    bool: Boolean!
    nullable: Nullable
    nullableList: [Nullable]!
    required: Required!
    requiredList: [Required!]!

}

scalar JSON

type Query {
    nullable: Nullable
    required: Required!
}
"#;

#[test]
fn nullable_field() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl("ext", SDL)
            .with_extension(StaticSelectionSetResolverExt::json(json!({ "int": "not an int" })))
            .build()
            .await;

        let response = engine
            .post(
                r#"
                query {
                    nullable {
                        int
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "nullable": {
              "int": null
            }
          },
          "errors": [
            {
              "message": "Invalid response from subgraph",
              "locations": [
                {
                  "line": 4,
                  "column": 25
                }
              ],
              "path": [
                "nullable",
                "int"
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
fn required_field() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl("ext", SDL)
            .with_extension(StaticSelectionSetResolverExt::json(json!({ "int": "not an int" })))
            .build()
            .await;

        let response = engine
            .post(
                r#"
                query {
                    required {
                        int
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": null,
          "errors": [
            {
              "message": "Invalid response from subgraph",
              "locations": [
                {
                  "line": 4,
                  "column": 25
                }
              ],
              "path": [
                "required",
                "int"
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
fn nullable_nested_object() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl("ext", SDL)
            .with_extension(StaticSelectionSetResolverExt::json(json!({
                "nullable": { "int": "not an int" }
            })))
            .build()
            .await;

        let response = engine
            .post(
                r#"
                query {
                    nullable {
                        nullable {
                            int
                        }
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "nullable": {
              "nullable": {
                "int": null
              }
            }
          },
          "errors": [
            {
              "message": "Invalid response from subgraph",
              "locations": [
                {
                  "line": 5,
                  "column": 29
                }
              ],
              "path": [
                "nullable",
                "nullable",
                "int"
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
fn required_nested_object() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl("ext", SDL)
            .with_extension(StaticSelectionSetResolverExt::json(json!({
                "required": { "int": "not an int" }
            })))
            .build()
            .await;

        let response = engine
            .post(
                r#"
                query {
                    nullable {
                        required {
                            int
                        }
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "nullable": null
          },
          "errors": [
            {
              "message": "Invalid response from subgraph",
              "locations": [
                {
                  "line": 5,
                  "column": 29
                }
              ],
              "path": [
                "nullable",
                "required",
                "int"
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
fn nullable_list_inner_nullable() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl("ext", SDL)
            .with_extension(StaticSelectionSetResolverExt::json(json!({
                "nullableList": [
                    { "int": "not an int" },
                    { "int": 42 }
                ]
            })))
            .build()
            .await;

        let response = engine
            .post(
                r#"
                query {
                    nullable {
                        nullableList {
                            int
                        }
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "nullable": {
              "nullableList": [
                {
                  "int": null
                },
                {
                  "int": 42
                }
              ]
            }
          },
          "errors": [
            {
              "message": "Invalid response from subgraph",
              "locations": [
                {
                  "line": 5,
                  "column": 29
                }
              ],
              "path": [
                "nullable",
                "nullableList",
                0,
                "int"
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
fn nullable_list_inner_required() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl("ext", SDL)
            .with_extension(StaticSelectionSetResolverExt::json(json!({
                "requiredList": [
                    { "int": "not an int" },
                    { "int": 42 }
                ]
            })))
            .build()
            .await;

        let response = engine
            .post(
                r#"
                query {
                    nullable {
                        requiredList {
                            int
                        }
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "nullable": {
              "requiredList": null
            }
          },
          "errors": [
            {
              "message": "Invalid response from subgraph",
              "locations": [
                {
                  "line": 5,
                  "column": 29
                }
              ],
              "path": [
                "nullable",
                "requiredList",
                0,
                "int"
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
fn required_list_inner_nullable() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl("ext", SDL)
            .with_extension(StaticSelectionSetResolverExt::json(json!({
                "nullableList": [
                    { "int": "not an int" },
                    { "int": 42 }
                ]
            })))
            .build()
            .await;

        let response = engine
            .post(
                r#"
                query {
                    required {
                        nullableList {
                            int
                        }
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "required": {
              "nullableList": [
                {
                  "int": null
                },
                {
                  "int": 42
                }
              ]
            }
          },
          "errors": [
            {
              "message": "Invalid response from subgraph",
              "locations": [
                {
                  "line": 5,
                  "column": 29
                }
              ],
              "path": [
                "required",
                "nullableList",
                0,
                "int"
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
fn required_list_inner_required() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl("ext", SDL)
            .with_extension(StaticSelectionSetResolverExt::json(json!({
                "requiredList": [
                     { "int": "not an int" },
                     { "int": 42 }
                ]
            })))
            .build()
            .await;

        let response = engine
            .post(
                r#"
                query {
                    required {
                        requiredList {
                            int
                        }
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": null,
          "errors": [
            {
              "message": "Invalid response from subgraph",
              "locations": [
                {
                  "line": 5,
                  "column": 29
                }
              ],
              "path": [
                "required",
                "requiredList",
                0,
                "int"
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
