use integration_tests::{cleanup_error, gateway::Gateway, runtime};

use super::EchoExt;

#[test]
fn list_coercion() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo", import: ["@echo", "@meta"])
                    @meta(value: "meta")

                scalar JSON

                type Query {
                    echo: JSON @echo(value: "something")
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                directive @meta(value: [String!]!) on SCHEMA
                directive @echo(value: [String!]!) on FIELD_DEFINITION
            "#,
            ))
            .build()
            .await;

        engine.post("query { echo }").await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "echo": {
          "schema": {
            "meta": {
              "value": [
                "meta"
              ]
            }
          },
          "directive": {
            "value": [
              "something"
            ]
          },
          "input": {}
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
                    @link(url: "echo", import: ["@echo", "@meta"])
                    @meta(value: "meta")

                scalar JSON

                type Query {
                    echo: JSON @echo(value: "something")
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                directive @meta(value: [[String!]]) on SCHEMA
                directive @echo(value: [[String!]]) on FIELD_DEFINITION
                "#,
            ))
            .build()
            .await;

        engine.post("query { echo }").await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "echo": {
          "schema": {
            "meta": {
              "value": [
                [
                  "meta"
                ]
              ]
            }
          },
          "directive": {
            "value": [
              [
                "something"
              ]
            ]
          },
          "input": {}
        }
      }
    }
    "#);
}

#[test]
fn incompatible_list_wrapping() {
    runtime().block_on(async move {
        // Invalid field directive
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo", import: ["@echo", "@meta"])

                scalar JSON

                type Query {
                    echo: JSON @echo(value: ["something"])
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(r#"
                directive @meta(value: [[String!]]) on SCHEMA
                directive @echo(value: [[String!]]) on FIELD_DEFINITION
            "#))
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.echo, for the extension 'echo-1.0.0' directive @echo: Found a String value where we expected a [String!] at path '.value.0'
        26 | {
        27 |   echo: JSON @extension__directive(graph: A, extension: ECHO, name: "echo", arguments: {value: ["something"]}) @join__field(graph: A)
                                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        28 | }
        "#);

        // Invalid schema directive
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo", import: ["@echo", "@meta"])
                    @meta(value: ["meta"])

                scalar JSON

                type Query {
                    echo: JSON
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(r#"
                directive @meta(value: [[String!]]) on SCHEMA
                directive @echo(value: [[String!]]) on FIELD_DEFINITION
            "#))
            .try_build()
            .await;

        insta::assert_snapshot!(cleanup_error(result.unwrap_err()), @r#"
        * At site subgraph named 'a', for the extension 'echo-1.0.0' directive @meta: Found a String value where we expected a [String!] at path '.value.0'
        36 | {
        37 |   ECHO @extension__link(url: "file:///tmp/XXXXXXXXXX/extensions/echo/v1.0.0", schemaDirectives: [{graph: A, name: "meta", arguments: {value: ["meta"]}}])
                                                                                                              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        38 | }
        "#);
    });
}
