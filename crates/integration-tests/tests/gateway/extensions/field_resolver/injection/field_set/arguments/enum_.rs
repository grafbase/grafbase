use crate::gateway::extensions::field_resolver::injection::field_set::arguments::DoubleEchoExt;
use integration_tests::{gateway::Gateway, runtime};

#[test]
fn valid_enum_value() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echoArgs", "@echo"])

                scalar JSON

                type Query {
                    field(value: EchoEnum!): JSON @echoArgs
                    echo: JSON @echo(fields: "field(value: VALUE)")
                }

                enum EchoEnum {
                    VALUE
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
            "value": "VALUE"
          }
        }
      }
    }
    "#);
}

#[test]
fn unknown_enum_value() {
    runtime().block_on(async move {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echoArgs", "@echo"])

                scalar JSON

                type Query {
                    field(value: EchoEnum!): JSON @echoArgs
                    echo: JSON @echo(fields: "field(value: UNKNOWN)")
                }

                enum EchoEnum {
                    VALUE
                }
                "#,
            )
            .with_extension(DoubleEchoExt)
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.echo, for the extension 'echo-1.0.0' directive @echo: Failed to coerce argument at path '.field': Found an unknown enum value 'UNKNOWN' for the enum EchoEnum at path '.value'
        See schema at 19:35:
        (graph: A, extension: ECHO, name: "echo", arguments: {fields: "field(value: UNKNOWN)"})
        "#);
    });
}

#[test]
fn invalid_enum_value() {
    runtime().block_on(async move {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echoArgs", "@echo"])

                scalar JSON

                type Query {
                    field(value: EchoEnum!): JSON @echoArgs
                    echo: JSON @echo(fields: "field(value: \"VALUE\")")
                }

                enum EchoEnum {
                    VALUE
                }
                "#,
            )
            .with_extension(DoubleEchoExt)
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.echo, for the extension 'echo-1.0.0' directive @echo: Failed to coerce argument at path '.field': Found a String value where we expected a EchoEnum enum value at path '.value'
        See schema at 19:35:
        (graph: A, extension: ECHO, name: "echo", arguments: {fields: "field(value: \"VALUE\")"})
        "#);

        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echoArgs", "@echo"])

                scalar JSON

                type Query {
                    field(value: EchoEnum!): JSON @echoArgs
                    echo: JSON @echo(fields: "field(value: 1)")
                }

                enum EchoEnum {
                    VALUE
                }
                "#,
            )
            .with_extension(DoubleEchoExt)
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.echo, for the extension 'echo-1.0.0' directive @echo: Failed to coerce argument at path '.field': Found a Integer value where we expected a EchoEnum enum value at path '.value'
        See schema at 19:35:
        (graph: A, extension: ECHO, name: "echo", arguments: {fields: "field(value: 1)"})
        "#);
    });
}
