use engine::Engine;
use integration_tests::{federation::EngineExt, runtime};

use super::EchoExt;

#[test]
fn valid_enum_value() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
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
            .with_extension(EchoExt {
                sdl: r#"
                    directive @meta(value: EchoEnum!) on SCHEMA
                    directive @echo(value: EchoEnum!) on FIELD_DEFINITION

                    enum EchoEnum {
                        VALUE
                    }
                "#,
            })
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
          }
        }
      }
    }
    "#);
}

#[test]
fn unknown_enum_value() {
    runtime().block_on(async move {
        // Invalid field directive
        let result = Engine::builder()
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
            .with_extension(EchoExt {
                sdl: r#"
                    directive @meta(value: EchoEnum!) on SCHEMA
                    directive @echo(value: EchoEnum!) on FIELD_DEFINITION

                    enum EchoEnum {
                        VALUE
                    }
                "#,
            })
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At Query.echo for the extension 'echo-1.0.0' directive named 'echo': Found an unknown enum value 'UNKNOWN' for the enum EchoEnum at path '.value'",
        )
        "#);

        // Invalid schema directive
        let result = Engine::builder()
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
            .with_extension(EchoExt {
                sdl: r#"
                    directive @meta(value: EchoEnum!) on SCHEMA
                    directive @echo(value: EchoEnum!) on FIELD_DEFINITION

                    enum EchoEnum {
                        VALUE
                    }
                "#,
            })
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At subgraph named 'a' for the extension 'echo-1.0.0' directive named 'meta': Found an unknown enum value 'UNKNOWN' for the enum EchoEnum at path '.value'",
        )
        "#);
    });
}

#[test]
fn invalid_enum_value() {
    runtime().block_on(async move {
        // Invalid field directive
        let result = Engine::builder()
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
            .with_extension(EchoExt {
                sdl: r#"
                    directive @meta(value: EchoEnum!) on SCHEMA
                    directive @echo(value: EchoEnum!) on FIELD_DEFINITION

                    enum EchoEnum {
                        VALUE
                    }
                "#,
            })
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At Query.echo for the extension 'echo-1.0.0' directive named 'echo': Found a String value where we expected a EchoEnum enum value at path '.value'",
        )
        "#);

        // Invalid schema directive
        let result = Engine::builder()
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
            .with_extension(EchoExt {
                sdl: r#"
                    directive @meta(value: EchoEnum!) on SCHEMA
                    directive @echo(value: EchoEnum!) on FIELD_DEFINITION

                    enum EchoEnum {
                        VALUE
                    }
                "#,
            })
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At subgraph named 'a' for the extension 'echo-1.0.0' directive named 'meta': Found a Integer value where we expected a EchoEnum enum value at path '.value'",
        )
        "#);
    });
}
