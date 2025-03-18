use async_graphql::ServerError;
use engine::Engine;
use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{federation::EngineExt, runtime};

#[test]
fn should_keep_original_error_code() {
    runtime().block_on(async move {
        let gateway = Engine::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                    type Query {
                        greeting: String
                    }
                    "#,
                )
                .with_resolver("Query", "greeting", {
                    let mut error = ServerError::new("My Error", None);
                    error.extensions.get_or_insert_default().set("code", "my-custom-code");
                    error
                })
                .into_subgraph("x"),
            )
            .build()
            .await;

        let response = gateway.post("{ greeting }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "greeting": null
          },
          "errors": [
            {
              "message": "My Error",
              "extensions": {
                "code": "my-custom-code"
              }
            }
          ]
        }
        "#);
    })
}

#[test]
fn should_keep_original_extensions_but_add_error_code_if_not_present() {
    runtime().block_on(async move {
        let gateway = Engine::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                    type Query {
                        greeting: String
                    }
                    "#,
                )
                .with_resolver("Query", "greeting", {
                    let mut error = ServerError::new("My Error", None);
                    error.extensions.get_or_insert_default().set(
                        "data",
                        graphql_mocks::dynamic::Value::from_json(serde_json::json!({
                            "a": [1, 2, {"custom": 3}]
                        }))
                        .unwrap(),
                    );
                    error
                })
                .into_subgraph("x"),
            )
            .build()
            .await;

        let response = gateway.post("{ greeting }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "greeting": null
          },
          "errors": [
            {
              "message": "My Error",
              "extensions": {
                "data": {
                  "a": [
                    1,
                    2,
                    {
                      "custom": 3
                    }
                  ]
                },
                "code": "SUBGRAPH_ERROR"
              }
            }
          ]
        }
        "#);
    })
}
