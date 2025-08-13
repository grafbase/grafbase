use graphql_mocks::EchoSchema;
use integration_tests::{gateway::Gateway, runtime};

#[test]
fn hooks_context() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_toml_config(
                r#"
                [extensions.auth-21.config]
                type = "error_with_context"
                "#,
            )
            .with_subgraph(EchoSchema::default())
            .with_extension("auth-21")
            .with_extension("hooks-21")
            .build()
            .await;

        let response = engine
            .post(r#"query { header(name: "token") }"#)
            .header("hooks-context", "I'm hooked!")
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "I'm hooked!",
              "extensions": {
                "code": "UNAUTHENTICATED"
              }
            }
          ]
        }
        "#);
    });
}
