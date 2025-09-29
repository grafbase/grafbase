use graphql_mocks::{EchoSchema, Schema as _};
use integration_tests::{gateway::Gateway, runtime};
use pretty_assertions::assert_eq;

#[test]
fn on_graphql_subgraph_request() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_toml_config(
                r#"
                [extensions.authz-21.config]
                context = "authz context"
                "#,
            )
            .with_extension("hooks-21")
            .with_extension("auth-21")
            .with_extension("authz-21")
            .with_subgraph(EchoSchema::default().with_sdl(
                r#"
                extend schema @link(url: "authz-21", import: ["@grant"])

                type Query {
                    header(name: String): String @grant
                }
                "#,
            ))
            .build()
            .await;

        let response = gateway
            .post(
                r#"query {
                    hooksContext: header(name: "hooks-context")
                    token: header(name: "token")
                    authorizationContext: header(name: "authorization-context")
                }"#,
            )
            .header("Authorization", "bearer")
            .header("hooks-context", "I'm hooked!")
            .await;
        insta::assert_snapshot!(response, @r#"
        {
          "data": {
            "hooksContext": "I'm hooked!",
            "token": "sdk21:bearer:default",
            "authorizationContext": "[\"authz context\"]"
          }
        }
        "#);
    });
}

#[test]
fn on_virtual_subgraph_request() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_toml_config(
                r#"
                [extensions.authz-21.config]
                context = "authz context"
                "#,
            )
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "authz-21", import: ["@grant"])
                    @link(url: "resolver-21", import: ["@echo"])

                scalar JSON

                type Query {
                    hooksContext: JSON @echo(header: "hooks-context") @grant
                    token: JSON @echo(header: "token") @grant
                    authorizationContext: JSON @echo(header: "authorization-context") @grant
                }
                "#,
            )
            .with_extension("hooks-21")
            .with_extension("auth-21")
            .with_extension("authz-21")
            .with_extension("resolver-21")
            .build()
            .await;

        let response = gateway
            .post(r#"query { hooksContext token authorizationContext }"#)
            .header("Authorization", "bearer")
            .header("hooks-context", "I'm hooked!")
            .await;
        insta::assert_snapshot!(response, @r#"
        {
          "data": {
            "hooksContext": "I'm hooked!",
            "token": "sdk21:bearer:default",
            "authorizationContext": "[\"authz context\"]"
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
            .header("hooks-context", "I'm hooked!")
            .await;
        assert_eq!(
            response.headers.get("hooks-context").and_then(|h| h.to_str().ok()),
            Some("I'm hooked!"),
        );
    });
}
