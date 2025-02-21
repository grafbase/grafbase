use crate::federation::extensions::injection::field_set::arguments::DoubleEchoExt;
use engine::Engine;
use integration_tests::{federation::EngineExt, runtime};

#[test]
fn list_coercion() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echoArgs", "@echo"])

                scalar JSON

                type Query {
                    field(value: [String!]!): JSON @echoArgs
                    echo: JSON @echo(fields: "field(value: \"something\")")
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
            "value": [
              "something"
            ]
          }
        }
      }
    }
    "#);
}

#[test]
fn list_list_coercion() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echoArgs", "@echo"])

                scalar JSON

                type Query {
                    field(value: [[String!]]!): JSON @echoArgs
                    echo: JSON @echo(fields: "field(value: \"something\")")
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
            "value": [
              [
                "something"
              ]
            ]
          }
        }
      }
    }
    "#);
}

#[test]
fn incompatible_list_wrapping() {
    runtime().block_on(async move {
        let result = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echoArgs", "@echo"])

                scalar JSON

                type Query {
                    field(value: [[String!]]!): JSON @echoArgs
                    echo: JSON @echo(fields: "field(value: [\"something\"])")
                }
                "#,
            )
            .with_extension(DoubleEchoExt)
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At Query.echo for the extension 'echo-1.0.0' directive @echo: Failed to coerce argument at path '.field': Found a String value where we expected a [String!] at path '.value.0'",
        )
        "#);
    });
}
