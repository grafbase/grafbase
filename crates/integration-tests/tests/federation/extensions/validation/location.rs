use engine::Engine;
use integration_tests::{federation::EngineExt, runtime};

use super::EchoExt;

#[test]
fn invalid_location() {
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
                    echo: JSON @meta
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                directive @meta on SCHEMA
                directive @echo on FIELD_DEFINITION
            "#,
            ))
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "Extension echo-1.0.0 directive @meta used in the wrong location FIELD_DEFINITION, expected one of: SCHEMA",
        )
        "#);

        // Invalid schema directive
        let result = Engine::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])
                    @echo

                scalar JSON

                type Query {
                    echo: JSON
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                directive @meta on SCHEMA
                directive @echo on FIELD_DEFINITION
                "#,
            ))
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "Extension echo-1.0.0 directive @echo used in the wrong location SCHEMA, expected one of: FIELD_DEFINITION",
        )
        "#);
    });
}
