use integration_tests::{cleanup_error, gateway::Gateway, runtime};

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
                    @link(url: "echo", import: ["@echo", "@meta"])

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

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.echo, for the extension 'echo-1.0.0' directive @echo: Unknown type 'EchoInput'
        26 | {
        27 |   echo: JSON @extension__directive(graph: A, extension: ECHO, name: "echo", arguments: {value: {a: 1}}) @join__field(graph: A)
                                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        28 | }
        "#);

        // Invalid schema directive
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo", import: ["@echo", "@meta"])
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

        let err = cleanup_error(result.unwrap_err());
        insta::assert_snapshot!(err, @r#"
        * At site subgraph named 'a', for the extension 'echo-1.0.0' directive @meta: Unknown type 'EchoInput'
        36 | {
        37 |   ECHO @extension__link(url: "file:///tmp/XXXXXXXXXX/extensions/echo/v1.0.0", schemaDirectives: [{graph: A, name: "meta", arguments: {value: {a: 1}}}])
                                                                                                              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        38 | }
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
                    @link(url: "echo", import: ["@echo", "@meta"])

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

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.echo, for the extension 'echo-1.0.0' directive @echo: Type 'EchoInput' is used for an input value but is not a scalar, input object or enum.
        26 | {
        27 |   echo: JSON @extension__directive(graph: A, extension: ECHO, name: "echo", arguments: {value: {a: 1}}) @join__field(graph: A)
                                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        28 | }
        "#);

        // Invalid schema directive
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo", import: ["@echo", "@meta"])
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

        insta::assert_snapshot!(cleanup_error(result.unwrap_err()), @r#"
        * At site subgraph named 'a', for the extension 'echo-1.0.0' directive @meta: Type 'EchoInput' is used for an input value but is not a scalar, input object or enum.
        36 | {
        37 |   ECHO @extension__link(url: "file:///tmp/XXXXXXXXXX/extensions/echo/v1.0.0", schemaDirectives: [{graph: A, name: "meta", arguments: {value: {a: 1}}}])
                                                                                                              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        38 | }
        "#);
    });
}
