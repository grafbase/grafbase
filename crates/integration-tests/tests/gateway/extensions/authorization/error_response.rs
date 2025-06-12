use engine::{ErrorResponse, GraphqlError};
use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{
    gateway::{AuthorizationExt, AuthorizationTestExtension, DynHookContext, Gateway},
    runtime,
};
use runtime::extension::{AuthorizationDecisions, QueryElement, TokenRef};

#[derive(Default)]
struct Failure;

#[async_trait::async_trait]
impl AuthorizationTestExtension for Failure {
    #[allow(clippy::manual_async_fn)]
    async fn authorize_query(
        &self,
        _wasm_context: DynHookContext,
        _headers: &tokio::sync::RwLock<http::HeaderMap>,
        _token: TokenRef<'_>,
        _elements_grouped_by_directive_name: Vec<(&str, Vec<QueryElement<'_, serde_json::Value>>)>,
    ) -> Result<AuthorizationDecisions, ErrorResponse> {
        Err(ErrorResponse {
            status: http::StatusCode::UNAUTHORIZED,
            errors: vec![GraphqlError::unauthorized()],
        })
    }
}

#[test]
fn can_return_error_response() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
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
            .with_extension(AuthorizationExt::new(Failure))
            .build()
            .await;

        let response = engine.post("query { greeting forbidden }").await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "Not authorized",
              "extensions": {
                "code": "UNAUTHORIZED"
              }
            }
          ]
        }
        "#);
        assert_eq!(response.status, 401);

        let sent = engine.drain_graphql_requests_sent_to_by_name("x");
        insta::assert_json_snapshot!(sent, @"[]")
    });
}
