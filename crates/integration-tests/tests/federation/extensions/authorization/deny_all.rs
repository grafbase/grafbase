use engine::{Engine, ErrorResponse, GraphqlError};
use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{
    federation::{EngineExt, TestExtension},
    runtime,
};
use runtime::{
    extension::{AuthorizationDecisions, QueryElement, TokenRef},
    hooks::DynHookContext,
};

use crate::federation::extensions::authorization::AuthorizationExt;

#[derive(Default)]
pub(super) struct DenyAll;

#[async_trait::async_trait]
impl TestExtension for DenyAll {
    #[allow(clippy::manual_async_fn)]
    async fn authorize_query(
        &self,
        _wasm_context: &DynHookContext,
        _headers: &tokio::sync::RwLock<http::HeaderMap>,
        _token: TokenRef<'_>,
        _elements_grouped_by_directive_name: Vec<(&str, Vec<QueryElement<'_, serde_json::Value>>)>,
    ) -> Result<AuthorizationDecisions, ErrorResponse> {
        Ok(AuthorizationDecisions::DenyAll(GraphqlError::unauthorized()))
    }
}

#[test]
fn can_deny_all() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

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
            .with_extension(AuthorizationExt::new(DenyAll))
            .build()
            .await;

        let response = engine.post("query { greeting forbidden }").await;
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
                  "column": 9
                }
              ],
              "path": [
                "greeting"
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
            "query": "query { __typename @skip(if: true) }",
            "operationName": null,
            "variables": {},
            "extensions": {}
          }
        ]
        "#)
    });
}
