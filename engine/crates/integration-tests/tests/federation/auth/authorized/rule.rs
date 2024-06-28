use std::collections::HashMap;

use integration_tests::federation::TestHooks;

use super::with_prepared_engine;

#[test]
fn authorized_hook_is_called() {
    let hooks = TestHooks::default()
        .on_gateway_request(|headers| {
            let mut ctx = HashMap::new();
            if let Some(client) = headers
                .get("x-grafbase-client-name")
                .and_then(|value| value.to_str().ok())
            {
                ctx.insert("client".to_string(), client.to_string());
            }
            Ok((ctx, headers))
        })
        .authorized(|ctx, inputs| {
            let maybe_error = if ctx.contains_key("client") {
                None
            } else {
                Some("Missing client".into())
            };
            Ok(vec![maybe_error; inputs.len()])
        });
    with_prepared_engine(hooks, |engine| async move {
        let response = engine
            .execute("query { check { grafbaseClientIsDefined } }")
            .by_client("hi", "")
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "check": {
              "grafbaseClientIsDefined": "You have properly set the x-grafbase-client-name header"
            }
          }
        }
        "###);

        let response = engine.execute("query { check { grafbaseClientIsDefined } }").await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Missing client",
              "path": [
                "check",
                "grafbaseClientIsDefined"
              ]
            }
          ]
        }
        "###);
    });
}
