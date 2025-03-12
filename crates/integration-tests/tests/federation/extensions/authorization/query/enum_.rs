use engine::Engine;
use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{federation::EngineExt, runtime};

use crate::federation::extensions::authorization::{AuthorizationExt, deny_some::DenySites};

#[test]
fn scalar() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

                enum State @auth {
                    ACTIVE
                    INACTIVE
                }

                enum Answer @auth {
                    YES
                    NO
                }

                type Query {
                    state: State
                    answer: Answer
                }
                "#,
                )
                .with_resolver("Query", "answer", serde_json::json!("YES"))
                .into_subgraph("x"),
            )
            .with_extension(AuthorizationExt::new(DenySites::query(vec!["State"])))
            .build()
            .await;

        let response = engine.post("query { state answer }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "state": null,
            "answer": "YES"
          },
          "errors": [
            {
              "message": "Unauthorized at query stage",
              "locations": [
                {
                  "line": 1,
                  "column": 9
                }
              ],
              "path": [
                "state"
              ],
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "#);

        let sent = engine.drain_graphql_requests_sent_to_by_name("x");
        insta::assert_json_snapshot!(sent, @r#"
        [
          {
            "query": "query { answer }",
            "operationName": null,
            "variables": {},
            "extensions": {}
          }
        ]
        "#)
    });
}
