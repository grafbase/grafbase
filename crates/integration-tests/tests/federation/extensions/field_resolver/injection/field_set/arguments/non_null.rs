use engine::Engine;

use crate::federation::extensions::field_resolver::injection::field_set::arguments::DoubleEchoExt;
use integration_tests::{federation::EngineExt, runtime};

#[test]
fn unexpected_null() {
    runtime().block_on(async move {
        let result = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echoArgs", "@echo"])

                scalar JSON

                type Query {
                    field(value: String!): JSON @echoArgs
                    echo: JSON @echo(fields: "field(value: null)")
                }
                "#,
            )
            .with_extension(DoubleEchoExt)
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At Query.echo for the extension 'echo-1.0.0' directive @echo: Failed to coerce argument at path '.field': Found a null where we expected a String! at path '.value'",
        )
        "#);
    });
}

#[test]
fn missing_required_argument() {
    runtime().block_on(async move {
        let result = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echoArgs", "@echo"])

                scalar JSON

                type Query {
                    field(value: String!): JSON @echoArgs
                    echo: JSON @echo(fields: "field")
                }
                "#,
            )
            .with_extension(DoubleEchoExt)
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At Query.echo for the extension 'echo-1.0.0' directive @echo: Failed to coerce argument at path '.field': Missing required argument named 'value'",
        )
        "#);
    });
}

#[test]
fn missing_nullable_argument() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echoArgs", "@echo"])

                scalar JSON

                type Query {
                    field(value: String): JSON @echoArgs
                    echo: JSON @echo(fields: "field")
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
          "field": {}
        }
      }
    }
    "#);
}

#[test]
fn distinguish_providing_null_from_not_present() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echoArgs", "@echo"])

                scalar JSON

                type Query {
                    field(a: String, b: String): JSON @echoArgs
                    echo: JSON @echo(fields: "field(a: null)")
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
            "a": null
          }
        }
      }
    }
    "#);
}
