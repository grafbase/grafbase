use std::sync::Arc;

use engine::Engine;
use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{
    federation::{EngineExt, TestExtension},
    runtime,
};
use runtime::{
    error::{ErrorResponse, PartialGraphqlError},
    extension::{AuthorizationDecisions, QueryElement},
};

use crate::federation::extensions::authorization::SimpleAuthExt;

#[derive(Default)]
pub(super) struct DenyAll;

#[async_trait::async_trait]
impl TestExtension for DenyAll {
    #[allow(clippy::manual_async_fn)]
    async fn authorize_query<'a>(
        &self,
        _ctx: Arc<engine::RequestContext>,
        _elements_grouped_by_directive_name: Vec<(&str, Vec<QueryElement<'_, serde_json::Value>>)>,
    ) -> Result<AuthorizationDecisions, ErrorResponse> {
        Ok(AuthorizationDecisions::DenyAll(PartialGraphqlError::new(
            "Not authorized",
            runtime::error::PartialErrorCode::Unauthorized,
        )))
    }
}

#[test]
fn can_deny_all() {
    let response = runtime().block_on(async move {
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
            .with_extension(SimpleAuthExt::new(DenyAll))
            .build()
            .await;

        engine.post("query { greeting forbidden }").await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "greeting": null,
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
        },
        {
          "message": "Not authorized",
          "locations": [
            {
              "line": 1,
              "column": 9
            }
          ],
          "path": [
            "greeting"
          ],
          "extensions": {
            "code": "UNAUTHORIZED"
          }
        }
      ]
    }
    "#);
}
