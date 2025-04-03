use integration_tests::{gateway::Gateway, runtime};

use super::EchoExt;

#[test]
fn unknown_type() {
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
                    echo: JSON @echo(value: { a: 1 })
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                directive @meta(value: EchoInput!) on SCHEMA
                directive @echo(value: EchoInput!) on FIELD_DEFINITION
                "#,
            ))
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At Query.echo for the extension 'echo-1.0.0' directive @echo: Unknown type 'EchoInput'",
        )
        "#);

        // Invalid schema directive
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])
                    @meta(value: { a: 1 })

                scalar JSON

                type Query {
                    echo: JSON
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                directive @meta(value: EchoInput!) on SCHEMA
                directive @echo(value: EchoInput!) on FIELD_DEFINITION
                "#,
            ))
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At subgraph named 'a' for the extension 'echo-1.0.0' directive @meta: Unknown type 'EchoInput'",
        )
        "#);
    });
}

#[test]
fn not_a_input_type() {
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
                    echo: JSON @echo(value: { a: 1 })
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(r#"
                directive @meta(value: EchoInput!) on SCHEMA
                directive @echo(value: EchoInput!) on FIELD_DEFINITION

                type EchoInput {
                    a: Int!
                }
            "#))
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At Query.echo for the extension 'echo-1.0.0' directive @echo: Type 'EchoInput' is used for an input value but is not a scalar, input object or enum.",
        )
        "#);

        // Invalid schema directive
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])
                    @meta(value: { a: 1 })

                scalar JSON

                type Query {
                    echo: JSON
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(r#"
                directive @meta(value: EchoInput!) on SCHEMA
                directive @echo(value: EchoInput!) on FIELD_DEFINITION

                type EchoInput {
                    a: Int!
                }
            "#))
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At subgraph named 'a' for the extension 'echo-1.0.0' directive @meta: Type 'EchoInput' is used for an input value but is not a scalar, input object or enum.",
        )
        "#);
    });
}
