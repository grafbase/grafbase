use integration_tests::{gateway::Gateway, runtime};

use crate::gateway::extensions::basic::GreetExt;

#[test]
fn invalid_link() {
    runtime().block_on(async move {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "greet", import: ["@greet"])

                scalar JSON

                type Query {
                    greet: JSON @greet
                }
                "#,
            )
            .with_extension(GreetExt::new().with_sdl(
                r#"
                    extend schema @link(ur: "http://specs.grafbase.com/grafbase")
                    directive @greet on FIELD_DEFINITION
                "#,
            ))
            .try_build()
            .await;

        insta::assert_snapshot!(result.unwrap_err(), @"For extension greet-1.0.0, failed to parse @link directive: Unknown argument `ur` in `@link` directive");
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
                    @link(url: "greet", import: ["@greet"])

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

        if let Err(err) = result {
            panic!("{err}")
        }
    });
}
