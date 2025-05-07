use crate::gateway::extensions::field_resolver::injection::field_set::arguments::DoubleEchoExt;
use integration_tests::{gateway::Gateway, runtime};

#[test]
fn missing_nullable_field() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
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
        let result = Gateway::builder()
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

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.echo, for the extension 'echo-1.0.0' directive @echo: Failed to coerce argument at path '.field': Found a null where we expected a String! at path '.input.value'
        See schema at 19:35:
        (graph: A, extension: ECHO, name: "echo", arguments: {fields: "field(input: {})"})
        "#);
    });
}

#[test]
fn too_many_fields() {
    runtime().block_on(async move {
        let result = Gateway::builder()
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

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.echo, for the extension 'echo-1.0.0' directive @echo: Failed to coerce argument at path '.field': Input object EchoInput does not have a field named 'other' at path '.input'
        See schema at 19:35:
        (graph: A, extension: ECHO, name: "echo", arguments: {fields: "field(input: { value: \"test\", other: 1 })"})
        "#);
    });
}

#[test]
fn not_an_object() {
    runtime().block_on(async move {
        let result = Gateway::builder()
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

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.echo, for the extension 'echo-1.0.0' directive @echo: Failed to coerce argument at path '.field': Found a List value where we expected a 'EchoInput' input object at path '.input'
        See schema at 19:35:
        (graph: A, extension: ECHO, name: "echo", arguments: {fields: "field(input: [])"})
        "#);
    });
}
