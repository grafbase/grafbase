use graphql_mocks::EchoSchema;
use integration_tests::{gateway::Gateway, runtime};
use pretty_assertions::assert_eq;

use crate::gateway::extensions::resolver::ResolverExt;

const HOOKS_CONTEXT: &str = "I'm hooked!";

#[test]
fn on_graphql_subgraph_request() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_extension("hooks-21")
            .with_subgraph(EchoSchema::default())
            .build()
            .await;

        let response = gateway
            .post(r#"query { header(name: "hooks-context") }"#)
            .header("hooks-context", HOOKS_CONTEXT)
            .await;
        insta::assert_snapshot!(response, @r#"
        {
          "data": {
            "header": "I'm hooked!"
          }
        }
        "#);
    });
}

#[test]
fn on_virtual_subgraph_request() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "resolver-1.0.0", import: ["@resolve"])

                scalar JSON

                type Query {
                    header: JSON @resolve
                }
                "#,
            )
            .with_extension(ResolverExt::echo_header("hooks-context"))
            .with_extension("hooks-21")
            .build()
            .await;

        let response = gateway
            .post(r#"query { header }"#)
            .header("hooks-context", HOOKS_CONTEXT)
            .await;
        insta::assert_snapshot!(response, @r#"
        {
          "data": {
            "header": "I'm hooked!"
          }
        }
        "#);
    });
}

#[test]
fn on_response() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_extension("hooks-21")
            .with_subgraph(EchoSchema::default())
            .build()
            .await;

        let response = gateway
            .post(r#"query { headers { name } }"#)
            .header("hooks-context", HOOKS_CONTEXT)
            .await;
        assert_eq!(
            response.headers.get("hooks-context").and_then(|h| h.to_str().ok()),
            Some(HOOKS_CONTEXT),
        );
    });
}
