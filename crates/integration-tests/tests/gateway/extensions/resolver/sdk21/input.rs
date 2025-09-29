use integration_tests::{gateway::Gateway, runtime};

#[test]
fn basic_request() {
    runtime().block_on(async {
        let engine = Gateway::builder()
            .with_toml_config(
                r#"
                [extensions.resolver-21.config]
                key = "test"
                "#,
            )
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "resolver-21", import: ["@echoInput"])

                scalar JSON

                type Query {
                    test(input: String): JSON @echoInput(data: {value: 1})
                }
                "#,
            )
            .with_extension("resolver-21")
            .build()
            .await;

        let response = engine.post("query { test(input: \"hi!\") }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "test": {
              "args": {
                "input": "hi!"
              },
              "config": {
                "key": "test"
              },
              "directive": {
                "data": {
                  "value": 1
                }
              }
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
                key = "test"
                "#,
            )
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "resolver-21", import: ["@echoInput"])

                scalar JSON

                type Subscription {
                    test(input: String): JSON @echoInput(data: {value: 1})
                }
                "#,
            )
            .with_extension("resolver-21")
            .build()
            .await;

        let response = engine
            .post("subscription { test(input: \"hi!\") }")
            .into_sse_stream()
            .await
            .collect()
            .await;
        insta::assert_json_snapshot!(response.messages, @r#"
        [
          {
            "data": {
              "test": {
                "args": {
                  "input": "hi!"
                },
                "config": {
                  "key": "test"
                },
                "directive": {
                  "data": {
                    "value": 1
                  }
                }
              }
            }
          },
          {
            "data": {
              "test": {
                "message": "This is a test message from the resolver extension."
              }
            }
          }
        ]
        "#);
    })
}
