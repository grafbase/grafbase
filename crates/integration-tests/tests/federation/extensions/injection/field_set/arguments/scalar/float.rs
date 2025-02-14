use engine::Engine;

use crate::federation::extensions::injection::field_set::arguments::DoubleEchoExt;
use integration_tests::{federation::EngineExt, runtime};

#[test]
fn valid_float() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echoArgs", "@echo"])

                scalar JSON

                type Query {
                    field(value: Float!): JSON @echoArgs
                    echo: JSON @echo(fields: "field(value: 780.123)")
                }
                "#,
            )
            .with_extension(DoubleEchoExt)
            .build()
            .await;

        engine.post("query { echo }").await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "echo": {
          "field": {
            "value": 780.123
          }
        }
      }
    }
    "#);
}

#[test]
fn int_to_float_conversion() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echoArgs", "@echo"])

                scalar JSON

                type Query {
                    field(value: Float!): JSON @echoArgs
                    echo: JSON @echo(fields: "field(value: 123)")
                }
                "#,
            )
            .with_extension(DoubleEchoExt)
            .build()
            .await;

        engine.post("query { echo }").await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "echo": {
          "field": {
            "value": 123.0
          }
        }
      }
    }
    "#);
}

#[test]
fn invalid_float() {
    runtime().block_on(async move {
        let result = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echoArgs", "@echo"])

                scalar JSON

                type Query {
                    field(value: Float!): JSON @echoArgs
                    echo: JSON @echo(fields: "field(value: \"7.123\")")
                }
                "#,
            )
            .with_extension(DoubleEchoExt)
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At Query.echo for the extension 'echo-1.0.0' directive @echo: Failed to coerce argument at path '.field': Found a String value where we expected a Float scalar at path '.value'",
        )
        "#);
    });
}
