use crate::gateway::extensions::field_resolver::injection::field_set::arguments::DoubleEchoExt;
use integration_tests::{gateway::Gateway, runtime};

#[test]
fn list_coercion() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
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
        let engine = Gateway::builder()
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
        let result = Gateway::builder()
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

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.echo, for the extension 'echo-1.0.0' directive @echo: Failed to coerce argument at path '.field': Found a String value where we expected a [String!] at path '.value.0'
        18 | {
        19 |   echo: JSON @extension__directive(graph: A, extension: ECHO, name: "echo", arguments: {fields: "field(value: [\"something\"])"}) @join__field(graph: A)
                                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        20 |   field(value: [[String!]]!): JSON @extension__directive(graph: A, extension: ECHO, name: "echoArgs", arguments: {}) @join__field(graph: A)
        "#);
    });
}
