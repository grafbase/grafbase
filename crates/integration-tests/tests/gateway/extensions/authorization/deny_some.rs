use engine::{ErrorCode, ErrorResponse, GraphqlError};
use engine_schema::DirectiveSite;
use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{
    gateway::{AuthorizationExt, AuthorizationTestExtension, Gateway},
    runtime,
};
use runtime::extension::{AuthorizationDecisions, QueryElement, TokenRef};

#[derive(Default)]
pub struct DenySites {
    pub query: Vec<&'static str>,
    pub response: Vec<&'static str>,
}

impl DenySites {
    pub fn query(query: impl IntoIterator<Item = &'static str>) -> Self {
        Self {
            query: query.into_iter().collect(),
            response: Vec::new(),
        }
    }

    pub fn response(response: impl IntoIterator<Item = &'static str>) -> Self {
        Self {
            query: Vec::new(),
            response: response.into_iter().collect(),
        }
    }
}

#[async_trait::async_trait]
impl AuthorizationTestExtension for DenySites {
    #[allow(clippy::manual_async_fn)]
    async fn authorize_query(
        &self,
        _ctx: engine::EngineRequestContext,
        _headers: &tokio::sync::RwLock<http::HeaderMap>,
        _token: TokenRef<'_>,
        elements_grouped_by_directive_name: Vec<(&str, Vec<QueryElement<'_, serde_json::Value>>)>,
    ) -> Result<(AuthorizationDecisions, Vec<u8>), ErrorResponse> {
        let mut element_to_error = Vec::new();
        let errors = vec![GraphqlError::new(
            "Unauthorized at query stage",
            ErrorCode::Unauthorized,
        )];

        let mut i = 0;
        for (_, elements) in elements_grouped_by_directive_name {
            for element in elements {
                if self.query.contains(&element.site.to_string().as_str()) {
                    element_to_error.push((i, 0));
                }
                i += 1;
            }
        }

        Ok((
            AuthorizationDecisions::DenySome {
                element_to_error,
                errors,
            },
            Vec::new(),
        ))
    }

    async fn authorize_response(
        &self,
        _ctx: engine::EngineOperationContext,
        _state: &[u8],
        _directive_name: &str,
        directive_site: DirectiveSite<'_>,
        _items: Vec<serde_json::Value>,
    ) -> Result<AuthorizationDecisions, GraphqlError> {
        if self.response.contains(&directive_site.to_string().as_str()) {
            Ok(AuthorizationDecisions::DenyAll(GraphqlError::new(
                "Unauthorized at response stage",
                ErrorCode::Unauthorized,
            )))
        } else {
            Ok(AuthorizationDecisions::GrantAll)
        }
    }
}

#[test]
fn can_deny_some() {
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
            .with_extension(AuthorizationExt::new(DenySites::query(vec!["Query.forbidden"])))
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
              "message": "Unauthorized at query stage",
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
