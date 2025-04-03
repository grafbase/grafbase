use integration_tests::{federation::Gateway, runtime};

use crate::federation::extensions::basic::GreetExt;

#[test]
fn invalid_link() {
    runtime().block_on(async move {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "greet-1.0.0", import: ["@greet"])

                scalar JSON

                type Query {
                    greet: JSON @greet
                }
                "#,
            )
            .with_extension(GreetExt::new().with_sdl( r#"
                    extend schema @link(ur: "http://specs.grafbase.com/grafbase")
                    directive @greet on FIELD_DEFINITION
                "#,
            ))
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "Could not read a @link directive used in the extension greet-1.0.0 GraphQL definitions: Unknown argument `ur` in `@link` directive",
        )
        "#);
    });
}

#[test]
fn valid_link() {
    runtime().block_on(async move {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "greet-1.0.0", import: ["@greet"])

                scalar JSON

                type Query {
                    greet: JSON @greet
                }
                "#,
            )
            .with_extension(GreetExt::new().with_sdl(
                r#"
                    extend schema @link(url: "http://specs.grafbase.com/grafbase")
                    directive @greet on FIELD_DEFINITION
                "#,
            ))
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @"None");
    });
}
