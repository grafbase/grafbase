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
    nullableList: [Nullable]!
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
fn missing_nullable_field() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl("ext", SDL)
            .with_extension(StaticSelectionSetResolverExt::json(json!({ "int": 1 })))
            .build()
            .await;

        let response = engine
            .post(
                r#"
                query {
                    nullable {
                        int
                        str
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "nullable": {
              "int": 1,
              "str": null
            }
          }
        }
        "#);
    })
}

#[test]
fn unknown_field() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl("ext", SDL)
            .with_extension(StaticSelectionSetResolverExt::json(
                json!({ "int": 1, "unknown": "Hi!" }),
            ))
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
              "int": 1
            }
          }
        }
        "#);
    })
}

#[test]
fn missing_required_field() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl("ext", SDL)
            .with_extension(StaticSelectionSetResolverExt::json(json!({ "int": 1 })))
            .build()
            .await;

        let response = engine
            .post(
                r#"
                query {
                    required {
                        int
                        str
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
                  "column": 25
                }
              ],
              "path": [
                "required",
                "str"
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
fn field_ordering_1() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl("ext", SDL)
            .with_extension(StaticSelectionSetResolverExt::json_bytes(
                br#"{
                    "str": "hello",
                    "int": 42,
                    "float": 3.14,
                    "bool": true
                }"#,
            ))
            .build()
            .await;

        let response = engine
            .post(
                r#"
                query {
                    nullable {
                        int
                        str
                        float
                        bool
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "nullable": {
              "int": 42,
              "str": "hello",
              "float": 3.14,
              "bool": true
            }
          }
        }
        "#);
    })
}

#[test]
fn field_ordering_2() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl("ext", SDL)
            .with_extension(StaticSelectionSetResolverExt::json_bytes(
                br#"{
                    "float": 3.14,
                    "str": "hello",
                    "bool": true,
                    "int": 42
                }"#,
            ))
            .build()
            .await;

        let response = engine
            .post(
                r#"
                query {
                    nullable {
                        int
                        str
                        float
                        bool
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "nullable": {
              "int": 42,
              "str": "hello",
              "float": 3.14,
              "bool": true
            }
          }
        }
        "#);
    })
}
