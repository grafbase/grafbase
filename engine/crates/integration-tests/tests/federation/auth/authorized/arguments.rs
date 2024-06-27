use integration_tests::federation::TestHooks;
use runtime::hooks::UserError;

use super::with_prepared_engine;

#[test]
fn arguments_are_provided() {
    let hooks = TestHooks::default().authorized(|_, _, inputs| {
        #[derive(serde::Deserialize)]
        struct Arguments {
            id: i64,
        }
        #[derive(serde::Deserialize)]
        struct Input {
            arguments: Arguments,
        }
        Ok(inputs
            .into_iter()
            .map(|input| match serde_json::from_str::<Input>(&input) {
                Ok(input) => {
                    if input.arguments.id < 100 {
                        Some("Unauthorized ID".into())
                    } else {
                        None
                    }
                }
                Err(err) => Some(UserError::from(err.to_string())),
            })
            .collect())
    });
    with_prepared_engine(hooks, |engine| async move {
        let response = engine
            .execute("query { check { sensitiveId(id: 791) } }")
            .by_client("hi", "")
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "check": {
              "sensitiveId": "You have access to the sensistive data"
            }
          }
        }
        "###);

        let response = engine.execute("query { check { sensitiveId(id: 0) } }").await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Unauthorized ID",
              "path": [
                "check",
                "sensitiveId"
              ]
            }
          ]
        }
        "###);
    });
}
