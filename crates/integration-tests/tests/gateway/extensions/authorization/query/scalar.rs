use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{
    gateway::{AuthorizationExt, Gateway},
    runtime,
};

use crate::gateway::extensions::authorization::deny_some::DenySites;

#[test]
fn scalar() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema @link(url: "authorization", import: ["@auth"])

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
            .with_extension(AuthorizationExt::new(DenySites::query(vec!["JSON"])))
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
              "message": "Unauthorized at query stage",
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
