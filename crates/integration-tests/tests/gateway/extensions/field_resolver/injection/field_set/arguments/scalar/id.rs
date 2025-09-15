use crate::gateway::extensions::field_resolver::injection::field_set::arguments::DoubleEchoExt;
use integration_tests::{gateway::Gateway, runtime};

#[test]
fn valid_id() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echoArgs", "@echo"])

                scalar JSON

                type Query {
                    field(value: ID!): JSON @echoArgs
                    echo: JSON @echo(fields: "field(value: \"xsdfwe\")")
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
            "value": "xsdfwe"
          }
        }
      }
    }
    "#);
}

#[test]
fn invalid_id() {
    runtime().block_on(async move {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echoArgs", "@echo"])

                scalar JSON

                type Query {
                    field(value: ID!): JSON @echoArgs
                    echo: JSON @echo(fields: "field(value: 7.123)")
                }
                "#,
            )
            .with_extension(DoubleEchoExt)
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.echo, for the extension 'echo-1.0.0' directive @echo: Failed to coerce argument at path '.field': Found a Float value where we expected a ID scalar at path '.value'
        26 | {
        27 |   echo: JSON @extension__directive(graph: A, extension: ECHO, name: "echo", arguments: {fields: "field(value: 7.123)"}) @join__field(graph: A)
                                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        28 |   field(value: ID!): JSON @extension__directive(graph: A, extension: ECHO, name: "echoArgs", arguments: {}) @join__field(graph: A)
        "#);
    });
}
