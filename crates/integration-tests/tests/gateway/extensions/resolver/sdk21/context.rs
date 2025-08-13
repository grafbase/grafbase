use integration_tests::{gateway::Gateway, runtime};

#[test]
fn basic_request() {
    runtime().block_on(async {
        let engine = Gateway::builder()
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
                    @link(url: "authz-21-1.0.0", import: ["@grant"])
                    @link(url: "resolver-21-1.0.0", import: ["@echoContext"])

                scalar JSON

                type Query {
                    test: JSON @echoContext @grant
                }
                "#,
            )
            .with_extension("hooks-21")
            .with_extension("auth-21")
            .with_extension("authz-21")
            .with_extension("resolver-21")
            .build()
            .await;

        let response = engine
            .post("query { test }")
            .header("hooks-context", "I'm hooked!")
            .header("Authorization", "bearer")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "test": {
              "authorization_context": [
                "authz context"
              ],
              "hooks_context": "I'm hooked!",
              "token": "sdk21:bearer:default"
            }
          }
        }
        "#);
    })
}

#[test]
fn basic_subscription() {
    runtime().block_on(async {
        let engine = Gateway::builder()
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
                    @link(url: "authz-21-1.0.0", import: ["@grant"])
                    @link(url: "resolver-21-1.0.0", import: ["@echoContext"])

                scalar JSON

                type Subscription {
                    test: JSON @echoContext @grant
                }
                "#,
            )
            .with_extension("hooks-21")
            .with_extension("auth-21")
            .with_extension("authz-21")
            .with_extension("resolver-21")
            .build()
            .await;

        let response = engine
            .post("subscription { test }")
            .header("hooks-context", "I'm hooked!")
            .header("Authorization", "bearer")
            .into_sse_stream()
            .await
            .collect()
            .await;
        insta::assert_json_snapshot!(response.messages, @r#"
        [
          {
            "data": {
              "test": {
                "authorization_context": [
                  "authz context"
                ],
                "hooks_context": "I'm hooked!",
                "token": "sdk21:bearer:default"
              }
            }
          }
        ]
        "#);
    })
}

#[test]
fn explicit_authz_context() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_toml_config(
                r#"
                [extensions.resolver-21.config]
                authorization_context = ["authz-21"]

                [extensions.authz-21.config]
                context = "authz context"
                "#,
            )
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "authz-21-1.0.0", import: ["@grant"])
                    @link(url: "resolver-21-1.0.0", import: ["@echoContext"])

                scalar JSON

                type Query {
                    test: JSON @echoContext @grant
                }
                "#,
            )
            .with_extension("hooks-21")
            .with_extension("auth-21")
            .with_extension("authz-21")
            .with_extension("resolver-21")
            .build()
            .await;

        let response = engine
            .post("query { test }")
            .header("hooks-context", "I'm hooked!")
            .header("Authorization", "bearer")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "test": {
              "authorization_context": [
                "authz context"
              ],
              "hooks_context": "I'm hooked!",
              "token": "sdk21:bearer:default"
            }
          }
        }
        "#);
    })
}
