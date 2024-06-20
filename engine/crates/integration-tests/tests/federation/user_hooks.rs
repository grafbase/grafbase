use std::collections::BTreeMap;

use engine_v2::Engine;
use graphql_mocks::{FakeGithubSchema, MockGraphQlServer};
use integration_tests::{
    federation::{GatewayV2Ext, UserHooksTest},
    runtime,
};
use runtime::user_hooks::{UserError, UserHookError};
use serde_json::Value;

#[test]
fn a_gateway_request_no_op() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;
        let user_hooks = UserHooksTest::default().on_gateway_request(Ok);

        let engine = Engine::builder()
            .with_user_hooks(user_hooks)
            .with_schema("schema", &github_mock)
            .await
            .finish()
            .await;

        engine.execute("query { serverVersion }").await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "serverVersion": "1"
      }
    }
    "###);
}

#[test]
fn a_gateway_callback_error() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let user_hooks = UserHooksTest::default().on_gateway_request(|_| {
            Err(UserHookError::User(UserError {
                extensions: BTreeMap::from([(String::from("foo"), Value::String(String::from("bar")))]),
                message: String::from("impossible error"),
            }))
        });

        let engine = Engine::builder()
            .with_user_hooks(user_hooks)
            .with_schema("schema", &github_mock)
            .await
            .finish()
            .await;

        engine.execute("query { serverVersion }").await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": null,
      "errors": [
        {
          "message": "impossible error",
          "extensions": {
            "foo": "bar"
          }
        }
      ]
    }
    "###);
}
