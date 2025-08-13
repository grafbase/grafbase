use integration_tests::{gateway::Gateway, runtime};

#[test]
fn basic_request() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_toml_config(
                r#"
                [extensions.resolver-21.config]
                type = "echo_context"
                "#,
            )
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "resolver-21-1.0.0", import: ["@resolve"])

                scalar JSON

                type Query {
                    test: JSON @resolve
                }
                "#,
            )
            .with_extension("hooks-21")
            .with_extension("auth-21")
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
                ""
              ],
              "hooks_context": "I'm hooked!",
              "token": "sdk19:bearer:default"
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
                [extensions.resolver-21.config]
                type = "echo_context"
                "#,
            )
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "resolver-21-1.0.0", import: ["@resolve"])

                scalar JSON

                type Subscription {
                    test: JSON @resolve
                }
                "#,
            )
            .with_extension("hooks-21")
            .with_extension("auth-21")
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
                  ""
                ],
                "hooks_context": "I'm hooked!",
                "token": "sdk19:bearer:default"
              }
            }
          }
        ]
        "#);
    })
}
