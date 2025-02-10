use engine::Engine;
use integration_tests::{federation::EngineExt, runtime};

use super::EchoExt;

#[test]
fn valid_string() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])
                    @meta(value: "8709")

                scalar JSON

                type Query {
                    echo: JSON @echo(value: "xsdfwe")
                }
                "#,
            )
            .with_extension(EchoExt {
                sdl: r#"
                    directive @meta(value: String!) on SCHEMA
                    directive @echo(value: String!) on FIELD_DEFINITION
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
              "value": "8709"
            }
          },
          "directive": {
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
        // Invalid field directive
        let result = Engine::builder()
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
            .with_extension(EchoExt {
                sdl: r#"
                    directive @meta(value: String!) on SCHEMA
                    directive @echo(value: String!) on FIELD_DEFINITION
                "#,
            })
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At Query.echo for the extension 'echo-1.0.0' directive named 'echo': Found a Float value where we expected a String scalar at path '.value'",
        )
        "#);

        // Invalid schema directive
        let result = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])
                    @meta(value: [])

                scalar JSON

                type Query {
                    echo: JSON
                }
                "#,
            )
            .with_extension(EchoExt {
                sdl: r#"
                    directive @meta(value: String!) on SCHEMA
                    directive @echo(value: String!) on FIELD_DEFINITION
                "#,
            })
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At subgraph named 'a' for the extension 'echo-1.0.0' directive named 'meta': Found a List value where we expected a String scalar at path '.value'",
        )
        "#);
    });
}
