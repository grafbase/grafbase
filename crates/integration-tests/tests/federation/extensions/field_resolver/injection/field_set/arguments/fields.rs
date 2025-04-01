use engine::Engine;

use crate::federation::extensions::field_resolver::injection::field_set::arguments::DoubleEchoExt;
use integration_tests::{federation::EngineExt, runtime};

#[test]
fn missing_nullable_field() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echoArgs", "@echo"])

                scalar JSON

                type Query {
                    field(input: EchoInput!): JSON @echoArgs
                    echo: JSON @echo(fields: "field(input: {})")
                }

                input EchoInput {
                    value: String
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
            "input": {}
          }
        }
      }
    }
    "#);
}

#[test]
fn missing_required_field() {
    runtime().block_on(async move {
        let result = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echoArgs", "@echo"])

                scalar JSON

                type Query {
                    field(input: EchoInput!): JSON @echoArgs
                    echo: JSON @echo(fields: "field(input: {})")
                }

                input EchoInput {
                    value: String!
                }
                "#,
            )
            .with_extension(DoubleEchoExt)
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At Query.echo for the extension 'echo-1.0.0' directive @echo: Failed to coerce argument at path '.field': Found a null where we expected a String! at path '.input.value'",
        )
        "#);
    });
}

#[test]
fn too_many_fields() {
    runtime().block_on(async move {
        let result = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echoArgs", "@echo"])

                scalar JSON

                type Query {
                    field(input: EchoInput!): JSON @echoArgs
                    echo: JSON @echo(fields: "field(input: { value: \"test\", other: 1 })")
                }

                input EchoInput {
                    value: String
                }
                "#,
            )
            .with_extension(DoubleEchoExt)
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At Query.echo for the extension 'echo-1.0.0' directive @echo: Failed to coerce argument at path '.field': Input object EchoInput does not have a field named 'other' at path '.input'",
        )
        "#);
    });
}

#[test]
fn not_an_object() {
    runtime().block_on(async move {
        let result = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echoArgs", "@echo"])

                scalar JSON

                type Query {
                    field(input: EchoInput!): JSON @echoArgs
                    echo: JSON @echo(fields: "field(input: [])")
                }

                input EchoInput {
                    value: String
                }
                "#,
            )
            .with_extension(DoubleEchoExt)
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At Query.echo for the extension 'echo-1.0.0' directive @echo: Failed to coerce argument at path '.field': Found a List value where we expected a 'EchoInput' input object at path '.input'",
        )
        "#);
    });
}
