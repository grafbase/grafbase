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

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At site Query.echo, for the extension 'echo-1.0.0' directive @echo: Failed to coerce argument at path '.field': Found a Float value where we expected a ID scalar at path '.value'. See schema at 19:35:\n(graph: A, extension: ECHO, name: \"echo\", arguments: {fields: \"field(value: 7.123)\"})",
        )
        "#);
    });
}
