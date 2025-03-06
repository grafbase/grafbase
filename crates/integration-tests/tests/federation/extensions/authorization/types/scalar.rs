use engine::Engine;
use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{federation::EngineExt, runtime};

use crate::federation::extensions::authorization::{SimpleAuthExt, deny_some::DenySites};

#[test]
fn scalar() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema @link(url: "simple-auth-1.0.0", import: ["@auth"])

                scalar JSON @auth
                scalar Public @auth

                type Query {
                    element: JSON
                    public: Public
                }
                "#,
                )
                .with_resolver("Query", "public", serde_json::json!("public"))
                .into_subgraph("x"),
            )
            .with_extension(SimpleAuthExt::new(DenySites(vec!["JSON"])))
            .build()
            .await;

        let response = engine.post("query { element }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "element": null
          },
          "errors": [
            {
              "message": "Not authorized",
              "locations": [
                {
                  "line": 1,
                  "column": 9
                }
              ],
              "path": [
                "element"
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
            "query": "query { __typename @skip(if: true) }",
            "operationName": null,
            "variables": {},
            "extensions": {}
          }
        ]
        "#)
    });
}
