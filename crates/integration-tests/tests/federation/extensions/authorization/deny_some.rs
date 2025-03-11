use std::sync::Arc;

use engine::{Engine, ErrorResponse, GraphqlError};
use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{
    federation::{EngineExt, TestExtension},
    runtime,
};
use runtime::extension::{AuthorizationDecisions, QueryElement};

use crate::federation::extensions::authorization::SimpleAuthExt;

#[derive(Default)]
pub(super) struct DenySites(pub Vec<&'static str>);

#[async_trait::async_trait]
impl TestExtension for DenySites {
    #[allow(clippy::manual_async_fn)]
    async fn authorize_query<'a>(
        &self,
        _ctx: Arc<engine::RequestContext>,
        elements_grouped_by_directive_name: Vec<(&str, Vec<QueryElement<'_, serde_json::Value>>)>,
    ) -> Result<AuthorizationDecisions, ErrorResponse> {
        let mut element_to_error = Vec::new();
        let errors = vec![GraphqlError::unauthorized()];

        let mut i = 0;
        for (_, elements) in elements_grouped_by_directive_name {
            for element in elements {
                if self.0.contains(&element.site.to_string().as_str()) {
                    element_to_error.push((i, 0));
                }
                i += 1;
            }
        }

        Ok(AuthorizationDecisions::DenySome {
            element_to_error,
            errors,
        })
    }
}

#[test]
fn can_deny_some() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema @link(url: "simple-auth-1.0.0", import: ["@auth"])

                type Query {
                    greeting: String @auth
                    forbidden: String @auth
                }
                "#,
                )
                .with_resolver("Query", "forbidden", serde_json::Value::String("Oh no!".to_owned()))
                .with_resolver("Query", "greeting", serde_json::Value::String("Hi!".to_owned()))
                .into_subgraph("x"),
            )
            .with_extension(SimpleAuthExt::new(DenySites(vec!["Query.forbidden"])))
            .build()
            .await;

        let response = engine.post("query { greeting forbidden }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "greeting": "Hi!",
            "forbidden": null
          },
          "errors": [
            {
              "message": "Not authorized",
              "locations": [
                {
                  "line": 1,
                  "column": 18
                }
              ],
              "path": [
                "forbidden"
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
            "query": "query { greeting }",
            "operationName": null,
            "variables": {},
            "extensions": {}
          }
        ]
        "#)
    });
}
