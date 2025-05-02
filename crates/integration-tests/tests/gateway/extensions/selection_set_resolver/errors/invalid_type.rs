use integration_tests::{gateway::Gateway, runtime};
use serde_json::json;

use crate::gateway::extensions::selection_set_resolver::StaticSelectionSetResolverExt;

const SDL: &str = r#"
extend schema
    @link(url: "static-1.0.0", import: ["@init"])
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
fn invalid_int() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl("ext", SDL)
            .with_extension(StaticSelectionSetResolverExt::json(json!({ "int": "19" })))
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
fn invalid_float() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl("ext", SDL)
            .with_extension(StaticSelectionSetResolverExt::json(json!({ "float": "19.5" })))
            .build()
            .await;

        let response = engine
            .post(
                r#"
                query {
                    nullable {
                        float
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "nullable": {
              "float": null
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
                "float"
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
fn invalid_boolean() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl("ext", SDL)
            .with_extension(StaticSelectionSetResolverExt::json(json!({ "bool": "true" })))
            .build()
            .await;

        let response = engine
            .post(
                r#"
                query {
                    nullable {
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
              "bool": null
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
                "bool"
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
fn invalid_string() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl("ext", SDL)
            .with_extension(StaticSelectionSetResolverExt::json(json!({ "str": 42 })))
            .build()
            .await;

        let response = engine
            .post(
                r#"
                query {
                    nullable {
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
              "str": null
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
fn invalid_id() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl("ext", SDL)
            .with_extension(StaticSelectionSetResolverExt::json(json!({ "id": 42 })))
            .build()
            .await;

        let response = engine
            .post(
                r#"
                query {
                    nullable {
                        id
                    }
                }
                "#,
            )
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "nullable": {
              "id": null
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
                "id"
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
fn invalid_list() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl("ext", SDL)
            .with_extension(StaticSelectionSetResolverExt::json(json!({
               "nullableList": "not a list"
            })))
            .build()
            .await;

        let response = engine
            .post(
                r#"
                query {
                    nullable {
                        nullableList {
                            id
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
              "nullableList": null
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
                "nullableList"
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
fn invalid_object() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_subgraph_sdl("ext", SDL)
            .with_extension(StaticSelectionSetResolverExt::json(json!({
                "nullable": "not an object",
            })))
            .build()
            .await;

        let response = engine
            .post(
                r#"
                query {
                    nullable {
                        nullable {
                            id
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
              "nullable": null
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
