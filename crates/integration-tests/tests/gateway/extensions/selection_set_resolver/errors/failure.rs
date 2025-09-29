use engine::{ErrorCode, GraphqlError};
use integration_tests::{gateway::Gateway, runtime};

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
fn nullable_field_invalid_json() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl("ext", SDL)
            .with_extension(StaticSelectionSetResolverExt::json_bytes(b"{]}"))
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
            "nullable": null
          },
          "errors": [
            {
              "message": "Invalid response from subgraph",
              "locations": [
                {
                  "line": 3,
                  "column": 21
                }
              ],
              "path": [
                "nullable"
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
fn required_field_invalid_json() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl("ext", SDL)
            .with_extension(StaticSelectionSetResolverExt::json_bytes(b"{]}"))
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
                  "line": 3,
                  "column": 21
                }
              ],
              "path": [
                "required"
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
fn nullable_field_subgraph_error() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl("ext", SDL)
            .with_extension(StaticSelectionSetResolverExt::error(GraphqlError::new(
                "oh no!",
                ErrorCode::BadRequest,
            )))
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
            "nullable": null
          },
          "errors": [
            {
              "message": "oh no!",
              "locations": [
                {
                  "line": 3,
                  "column": 21
                }
              ],
              "path": [
                "nullable"
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
fn required_field_subgraph_error() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl("ext", SDL)
            .with_extension(StaticSelectionSetResolverExt::error(GraphqlError::new(
                "oh no!",
                ErrorCode::BadRequest,
            )))
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
              "message": "oh no!",
              "locations": [
                {
                  "line": 3,
                  "column": 21
                }
              ],
              "path": [
                "required"
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
