use integration_tests::{gateway::Gateway, runtime};

use super::EchoExt;

#[test]
fn valid_enum_value() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])
                    @meta(value: VALUE)

                scalar JSON

                type Query {
                    echo: JSON @echo(value: VALUE)
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                directive @meta(value: EchoEnum!) on SCHEMA
                directive @echo(value: EchoEnum!) on FIELD_DEFINITION

                enum EchoEnum {
                    VALUE
                }
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
              "value": "VALUE"
            }
          },
          "directive": {
            "value": "VALUE"
          },
          "input": {}
        }
      }
    }
    "#);
}

#[test]
fn unknown_enum_value() {
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
                    echo: JSON @echo(value: UNKNOWN)
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(r#"
                directive @meta(value: EchoEnum!) on SCHEMA
                directive @echo(value: EchoEnum!) on FIELD_DEFINITION

                enum EchoEnum {
                    VALUE
                }
            "#))
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.echo, for the extension 'echo-1.0.0' directive @echo: Found an unknown enum value 'UNKNOWN' for the enum EchoEnum at path '.value'
        See schema at 19:35:
        (graph: A, extension: ECHO, name: "echo", arguments: {value: UNKNOWN})
        "#);

        // Invalid schema directive
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])
                    @meta(value: UNKNOWN)

                scalar JSON

                type Query {
                    echo: JSON
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(r#"
                directive @meta(value: EchoEnum!) on SCHEMA
                directive @echo(value: EchoEnum!) on FIELD_DEFINITION

                enum EchoEnum {
                    VALUE
                }
            "#))
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site subgraph named 'a', for the extension 'echo-1.0.0' directive @meta: Found an unknown enum value 'UNKNOWN' for the enum EchoEnum at path '.value'
        See schema at 29:97:
        {graph: A, name: "meta", arguments: {value: UNKNOWN}}
        "#);
    });
}

#[test]
fn invalid_enum_value() {
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
                    echo: JSON @echo(value: "VALID")
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(r#"
                directive @meta(value: EchoEnum!) on SCHEMA
                directive @echo(value: EchoEnum!) on FIELD_DEFINITION

                enum EchoEnum {
                    VALUE
                }
            "#))
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.echo, for the extension 'echo-1.0.0' directive @echo: Found a String value where we expected a EchoEnum enum value at path '.value'
        See schema at 19:35:
        (graph: A, extension: ECHO, name: "echo", arguments: {value: "VALID"})
        "#);

        // Invalid schema directive
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])
                    @meta(value: 1)

                scalar JSON

                type Query {
                    echo: JSON
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(r#"
                directive @meta(value: EchoEnum!) on SCHEMA
                directive @echo(value: EchoEnum!) on FIELD_DEFINITION

                enum EchoEnum {
                    VALUE
                }
            "#))
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site subgraph named 'a', for the extension 'echo-1.0.0' directive @meta: Found a Integer value where we expected a EchoEnum enum value at path '.value'
        See schema at 29:97:
        {graph: A, name: "meta", arguments: {value: 1}}
        "#);
    });
}
