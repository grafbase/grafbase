use crate::federation::extensions::field_resolver::injection::field_set::arguments::DoubleEchoExt;
use integration_tests::{federation::Gateway, runtime};

#[test]
fn valid_string() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echoArgs", "@echo"])

                scalar JSON

                type Query {
                    field(value: String!): JSON @echoArgs
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
fn invalid_string() {
    runtime().block_on(async move {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echoArgs", "@echo"])

                scalar JSON

                type Query {
                    field(value: String!): JSON @echoArgs
                    echo: JSON @echo(fields: "field(value: 7.123)")
                }
                "#,
            )
            .with_extension(DoubleEchoExt)
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At Query.echo for the extension 'echo-1.0.0' directive @echo: Failed to coerce argument at path '.field': Found a Float value where we expected a String scalar at path '.value'",
        )
        "#);
    });
}
