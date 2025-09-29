use crate::gateway::extensions::field_resolver::injection::field_set::arguments::DoubleEchoExt;
use integration_tests::{gateway::Gateway, runtime};

#[test]
fn unexpected_null() {
    runtime().block_on(async move {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo", import: ["@echoArgs", "@echo"])

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

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.echo, for the extension 'echo-1.0.0' directive @echo: Failed to coerce argument at path '.field': Found a null where we expected a String! at path '.value'
        26 | {
        27 |   echo: JSON @extension__directive(graph: A, extension: ECHO, name: "echo", arguments: {fields: "field(value: null)"}) @join__field(graph: A)
                                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        28 |   field(value: String!): JSON @extension__directive(graph: A, extension: ECHO, name: "echoArgs", arguments: {}) @join__field(graph: A)
        "#);
    });
}

#[test]
fn missing_required_argument() {
    runtime().block_on(async move {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo", import: ["@echoArgs", "@echo"])

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

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.echo, for the extension 'echo-1.0.0' directive @echo: Failed to coerce argument at path '.field': Missing required argument named 'value'
        26 | {
        27 |   echo: JSON @extension__directive(graph: A, extension: ECHO, name: "echo", arguments: {fields: "field"}) @join__field(graph: A)
                                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        28 |   field(value: String!): JSON @extension__directive(graph: A, extension: ECHO, name: "echoArgs", arguments: {}) @join__field(graph: A)
        "#);
    });
}

#[test]
fn missing_nullable_argument() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo", import: ["@echoArgs", "@echo"])

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
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo", import: ["@echoArgs", "@echo"])

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
