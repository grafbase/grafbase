use integration_tests::{gateway::Gateway, runtime};

use super::EchoExt;

#[test]
fn valid_boolean() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])
                    @meta(value: true)

                scalar JSON

                type Query {
                    echo: JSON @echo(value: false)
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                directive @meta(value: Boolean!) on SCHEMA
                directive @echo(value: Boolean!) on FIELD_DEFINITION
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
              "value": true
            }
          },
          "directive": {
            "value": false
          },
          "input": {}
        }
      }
    }
    "#);
}

#[test]
fn invalid_boolean() {
    runtime().block_on(async move {
        // Invalid field directive
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])

                scalar JSON

                type Query {
                    echo: JSON @echo(value: 7.123)
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(r#"
                directive @meta(value: Boolean!) on SCHEMA
                directive @echo(value: Boolean!) on FIELD_DEFINITION
            "#))
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At site Query.echo, for the extension 'echo-1.0.0' directive @echo: Found a Float value where we expected a Boolean scalar at path '.value'. See schema at 19:35:\n(graph: A, extension: ECHO, name: \"echo\", arguments: {value: 7.123})",
        )
        "#);

        // Invalid schema directive
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])
                    @meta(value: "test")

                scalar JSON

                type Query {
                    echo: JSON
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(r#"
                directive @meta(value: Boolean!) on SCHEMA
                directive @echo(value: Boolean!) on FIELD_DEFINITION
            "#))
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At site subgraph named 'a', for the extension 'echo-1.0.0' directive @meta: Found a String value where we expected a Boolean scalar at path '.value'. See schema at 29:97:\n{graph: A, name: \"meta\", arguments: {value: \"test\"}}",
        )
        "#);
    });
}
