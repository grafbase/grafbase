use integration_tests::{gateway::Gateway, runtime};

use super::EchoExt;

#[test]
fn validate() {
    runtime().block_on(async move {
        let ext = EchoExt::with_sdl(r#"
            directive @meta(value: Test!) on SCHEMA
            directive @echo(value: Test!) on FIELD_DEFINITION

            input Test @oneOf {
                a: Int
                b: String
            }
        "#);

        //
        // { a: 1 }
        //

        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])

                type Query {
                    echo: Int @echo(value: {a: 1})
                }
                "#,
            )
            .with_extension(ext)
            .try_build()
            .await;
        if let Err(err) = result {
            panic!("{err}")
        }

        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])
                    @meta(value: {a : 1})

                type Query {
                    echo: Int
                }
                "#,
            )
            .with_extension(ext)
            .try_build()
            .await;
        if let Err(err) = result {
            panic!("{err}")
        }

        //
        // { b: "1" }
        //

        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])

                type Query {
                    echo: Int @echo(value: {b: "1"})
                }
                "#,
            )
            .with_extension(ext)
            .try_build()
            .await;
        if let Err(err) = result {
            panic!("{err}")
        }

        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])
                    @meta(value: {b: "1"})

                type Query {
                    echo: Int
                }
                "#,
            )
            .with_extension(ext)
            .try_build()
            .await;
        if let Err(err) = result {
            panic!("{err}")
        }

        //
        // {}
        //

        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])

                type Query {
                    echo: Int @echo(value: {})
                }
                "#,
            )
            .with_extension(ext)
            .try_build()
            .await;
        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.echo, for the extension 'echo-1.0.0' directive @echo: Exactly one field must be provided for Test with @oneOf: No field was provided at path '.value'
        See schema at 17:34:
        (graph: A, extension: ECHO, name: "echo", arguments: {value: {}})
        "#);

        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])
                    @meta(value: {})

                type Query {
                    echo: Int
                }
                "#,
            )
            .with_extension(ext)
            .try_build()
            .await;
        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site subgraph named 'a', for the extension 'echo-1.0.0' directive @meta: Exactly one field must be provided for Test with @oneOf: No field was provided at path '.value'
        See schema at 27:97:
        {graph: A, name: "meta", arguments: {value: {}}}
        "#);

        //
        // { a: 1, b: "1" }
        //

        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])

                type Query {
                    echo: Int @echo(value: {a: 1, b: "1"})
                }
                "#,
            )
            .with_extension(ext)
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site Query.echo, for the extension 'echo-1.0.0' directive @echo: Exactly one field must be provided for Test with @oneOf: 2 fields (a,b) were provided at path '.value'
        See schema at 17:34:
        (graph: A, extension: ECHO, name: "echo", arguments: {value: {a: 1, b: "1"}})
        "#);
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo", "@meta"])
                    @meta(value: {a: 1, b: "1"})

                type Query {
                    echo: Int
                }
                "#,
            )
            .with_extension(ext)
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        At site subgraph named 'a', for the extension 'echo-1.0.0' directive @meta: Exactly one field must be provided for Test with @oneOf: 2 fields (a,b) were provided at path '.value'
        See schema at 27:97:
        {graph: A, name: "meta", arguments: {value: {a: 1, b: "1"}}}
        "#);
    });
}
