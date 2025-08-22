use integration_tests::{cleanup_error, gateway::Gateway, runtime};

use super::EchoExt;

#[test]
fn valid_int() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])
                    @meta(value: 9089)

                scalar JSON

                type Query {
                    echo: JSON @echo(value: 9089)
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                directive @meta(value: Int!) on SCHEMA
                directive @echo(value: Int!) on FIELD_DEFINITION
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
              "value": 9089
            }
          },
          "directive": {
            "value": 9089
          },
          "input": {}
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
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])
                    @meta(value: 1.0)

                scalar JSON

                type Query {
                    echo: JSON @echo(value: 7.0)
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                directive @meta(value: Int!) on SCHEMA
                directive @echo(value: Int!) on FIELD_DEFINITION
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
              "value": 1
            }
          },
          "directive": {
            "value": 7
          },
          "input": {}
        }
      }
    }
    "#);
}

#[test]
fn invalid_int() {
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
                    echo: JSON @echo(value: 9223372036854775807)
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(r#"
                directive @meta(value: Int!) on SCHEMA
                directive @echo(value: Int!) on FIELD_DEFINITION
            "#))
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.echo, for the extension 'echo-1.0.0' directive @echo: Found value 9223372036854775807 which cannot be coerced into a Int scalar at path '.value'
        18 | {
        19 |   echo: JSON @extension__directive(graph: A, extension: ECHO, name: "echo", arguments: {value: 9223372036854775807}) @join__field(graph: A)
                                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        20 | }
        "#);

        // Invalid schema directive
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])
                    @meta(value: 79.123)

                scalar JSON

                type Query {
                    echo: JSON
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(r#"
                directive @meta(value: Int!) on SCHEMA
                directive @echo(value: Int!) on FIELD_DEFINITION
            "#))
            .try_build()
            .await;

        insta::assert_snapshot!(cleanup_error(result.unwrap_err()), @r#"
        * At site subgraph named 'a', for the extension 'echo-1.0.0' directive @meta: Found a Float value where we expected a Int scalar at path '.value'
        28 | {
        29 |   ECHO @extension__link(url: "file:///tmp/XXXXXXXXXX/extensions/echo-1.0.0", schemaDirectives: [{graph: A, name: "meta", arguments: {value: 79.123}}])
                                                                                                             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        30 | }
        "#);
    });
}
