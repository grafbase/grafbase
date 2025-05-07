use integration_tests::{gateway::Gateway, runtime};

use super::EchoExt;

#[test]
fn valid_float() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])
                    @meta(value: 780.123)

                scalar JSON

                type Query {
                    echo: JSON @echo(value: -78901.23)
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                directive @meta(value: Float!) on SCHEMA
                directive @echo(value: Float!) on FIELD_DEFINITION
            "#,
            ))
            .build()
            .await;

        engine.post("query { echo }").await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "echo": {
          "schema": {
            "meta": {
              "value": 780.123
            }
          },
          "directive": {
            "value": -78901.23
          },
          "input": {}
        }
      }
    }
    "#);
}

#[test]
fn int_to_float_conversion() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])
                    @meta(value: 109879)

                scalar JSON

                type Query {
                    echo: JSON @echo(value: -123)
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                directive @meta(value: Float!) on SCHEMA
                directive @echo(value: Float!) on FIELD_DEFINITION
            "#,
            ))
            .build()
            .await;

        engine.post("query { echo }").await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "echo": {
          "schema": {
            "meta": {
              "value": 109879.0
            }
          },
          "directive": {
            "value": -123.0
          },
          "input": {}
        }
      }
    }
    "#);
}

#[test]
fn invalid_float() {
    runtime().block_on(async move {
        // Invalid field directive
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])

                scalar JSON

                type Query {
                    echo: JSON @echo(value: {})
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(r#"
                directive @meta(value: Float!) on SCHEMA
                directive @echo(value: Float!) on FIELD_DEFINITION
            "#))
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.echo, for the extension 'echo-1.0.0' directive @echo: Found a Object value where we expected a Float scalar at path '.value'
        See schema at 19:35:
        (graph: A, extension: ECHO, name: "echo", arguments: {value: {}})
        "#);

        // Invalid schema directive
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])
                    @meta(value: "79.123")

                scalar JSON

                type Query {
                    echo: JSON
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(r#"
                directive @meta(value: Float!) on SCHEMA
                directive @echo(value: Float!) on FIELD_DEFINITION
            "#))
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site subgraph named 'a', for the extension 'echo-1.0.0' directive @meta: Found a String value where we expected a Float scalar at path '.value'
        See schema at 29:97:
        {graph: A, name: "meta", arguments: {value: "79.123"}}
        "#);
    });
}
