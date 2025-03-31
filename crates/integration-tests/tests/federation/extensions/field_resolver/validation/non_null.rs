use engine::Engine;
use integration_tests::{federation::EngineExt, runtime};

use super::EchoExt;

#[test]
fn unexpected_null() {
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
                    echo: JSON @echo(value: null)
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(r#"
                directive @meta(value: String!) on SCHEMA
                directive @echo(value: String!) on FIELD_DEFINITION
            "#))
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At Query.echo for the extension 'echo-1.0.0' directive @echo: Found a null where we expected a String! at path '.value'",
        )
        "#);

        // Invalid schema directive
        let result = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])
                    @meta(value: null)

                scalar JSON

                type Query {
                    echo: JSON
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(r#"
                directive @meta(value: String!) on SCHEMA
                directive @echo(value: String!) on FIELD_DEFINITION
            "#))
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At subgraph named 'a' for the extension 'echo-1.0.0' directive @meta: Found a null where we expected a String! at path '.value'",
        )
        "#);
    });
}

#[test]
fn missing_required_argument() {
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
                    echo: JSON @echo
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(r#"
                directive @meta(value: String!) on SCHEMA
                directive @echo(value: String!) on FIELD_DEFINITION
            "#))
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At Query.echo for the extension 'echo-1.0.0' directive @echo: Missing required argument named 'value'",
        )
        "#);

        // Invalid schema directive
        let result = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])
                    @meta

                scalar JSON

                type Query {
                    echo: JSON
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(r#"
                directive @meta(value: String!) on SCHEMA
                directive @echo(value: String!) on FIELD_DEFINITION
            "#))
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At subgraph named 'a' for the extension 'echo-1.0.0' directive @meta: Missing required argument named 'value'",
        )
        "#);
    });
}

#[test]
fn missing_nullable_argument() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])
                    @meta

                scalar JSON

                type Query {
                    echo: JSON @echo
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                directive @meta(value: String) on SCHEMA
                directive @echo(value: String) on FIELD_DEFINITION
                "#,
            ))
            .build()
            .await;

        let response = engine.post("query { echo }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "echo": {
              "schema": {
                "meta": {}
              },
              "directive": {},
              "input": {}
            }
          }
        }
        "#);
    });
}

#[test]
fn distinquish_providing_null_from_not_present_at_all() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])
                    @meta(a: null)

                scalar JSON

                type Query {
                    echo: JSON @echo(a: null)
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                directive @meta(a: String, b: String) on SCHEMA
                directive @echo(a: String, b: String) on FIELD_DEFINITION
                "#,
            ))
            .build()
            .await;

        let response = engine.post("query { echo }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "echo": {
              "schema": {
                "meta": {
                  "a": null
                }
              },
              "directive": {
                "a": null
              },
              "input": {}
            }
          }
        }
        "#);
    });
}
