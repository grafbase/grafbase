use crate::gateway::extensions::field_resolver::injection::field_set::arguments::DoubleEchoExt;
use integration_tests::{gateway::Gateway, runtime};

#[test]
fn valid_int() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo", import: ["@echoArgs", "@echo"])

                scalar JSON

                type Query {
                    field(value: Int!): JSON @echoArgs
                    echo: JSON @echo(fields: "field(value: 9089)")
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
            "value": 9089
          }
        }
      }
    }
    "#);
}

#[test]
fn float_to_int_coercion() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo", import: ["@echoArgs", "@echo"])

                scalar JSON

                type Query {
                    field(value: Int!): JSON @echoArgs
                    echo: JSON @echo(fields: "field(value: 7.0)")
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
            "value": 7
          }
        }
      }
    }
    "#);
}

#[test]
fn invalid_int() {
    runtime().block_on(async move {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo", import: ["@echoArgs", "@echo"])

                scalar JSON

                type Query {
                    field(value: Int!): JSON @echoArgs
                    echo: JSON @echo(fields: "field(value: 9223372036854775807)")
                }
                "#,
            )
            .with_extension(DoubleEchoExt)
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.echo, for the extension 'echo-1.0.0' directive @echo: Failed to coerce argument at path '.field': Found value 9223372036854775807 which cannot be coerced into a Int scalar at path '.value'
        26 | {
        27 |   echo: JSON @extension__directive(graph: A, extension: ECHO, name: "echo", arguments: {fields: "field(value: 9223372036854775807)"}) @join__field(graph: A)
                                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        28 |   field(value: Int!): JSON @extension__directive(graph: A, extension: ECHO, name: "echoArgs", arguments: {}) @join__field(graph: A)
        "#);

        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo", import: ["@echoArgs", "@echo"])

                scalar JSON

                type Query {
                    field(value: Int!): JSON @echoArgs
                    echo: JSON @echo(fields: "field(value: 79.123)")
                }
                "#,
            )
            .with_extension(DoubleEchoExt)
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.echo, for the extension 'echo-1.0.0' directive @echo: Failed to coerce argument at path '.field': Found a Float value where we expected a Int scalar at path '.value'
        26 | {
        27 |   echo: JSON @extension__directive(graph: A, extension: ECHO, name: "echo", arguments: {fields: "field(value: 79.123)"}) @join__field(graph: A)
                                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        28 |   field(value: Int!): JSON @extension__directive(graph: A, extension: ECHO, name: "echoArgs", arguments: {}) @join__field(graph: A)
        "#);
    });
}
