use integration_tests::{gateway::Gateway, runtime};

use super::EchoExt;

#[test]
fn default_values() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])
                    @meta

                scalar JSON

                type Query {
                    echo: JSON @echo
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                directive @meta(a: EchoInput! = { x: 3 }, b: Int) on SCHEMA
                directive @echo(a: EchoInput! = { x: 3 }, b: Int) on FIELD_DEFINITION

                input EchoInput {
                    x: Int!
                    y: String! = "default"
                }
                "#,
            ))
            .build()
            .await;

        let response = engine.post("query { echo }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "echo": {
              "schema": {
                "meta": {
                  "a": {
                    "x": 3,
                    "y": "default"
                  }
                }
              },
              "directive": {
                "a": {
                  "x": 3,
                  "y": "default"
                }
              },
              "input": {}
            }
          }
        }
        "#);
    });
}

#[test]
fn default_values_partial_override() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])
                    @meta(a: { x: 5 })

                scalar JSON

                type Query {
                    echo: JSON @echo(a: { x: 5 })
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                directive @meta(a: EchoInput! = { x: 3 }, b: Int) on SCHEMA
                directive @echo(a: EchoInput! = { x: 3 }, b: Int) on FIELD_DEFINITION

                input EchoInput {
                    x: Int!
                    y: String! = "default"
                }
            "#,
            ))
            .build()
            .await;

        let response = engine.post("query { echo }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "echo": {
              "schema": {
                "meta": {
                  "a": {
                    "x": 5,
                    "y": "default"
                  }
                }
              },
              "directive": {
                "a": {
                  "x": 5,
                  "y": "default"
                }
              },
              "input": {}
            }
          }
        }
        "#);
    });
}

#[test]
fn default_values_override() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])
                    @meta(a: { x: 5, y: "override" })

                scalar JSON

                type Query {
                    echo: JSON @echo(a: { x: 5, y: "override" })
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                directive @meta(a: EchoInput! = { x: 3 }, b: Int) on SCHEMA
                directive @echo(a: EchoInput! = { x: 3 }, b: Int) on FIELD_DEFINITION

                input EchoInput {
                    x: Int!
                    y: String! = "default"
                }
            "#,
            ))
            .build()
            .await;

        let response = engine.post("query { echo }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "echo": {
              "schema": {
                "meta": {
                  "a": {
                    "x": 5,
                    "y": "override"
                  }
                }
              },
              "directive": {
                "a": {
                  "x": 5,
                  "y": "override"
                }
              },
              "input": {}
            }
          }
        }
        "#);
    });
}
