use engine::{ErrorCode, ErrorResponse, GraphqlError};
use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{
    gateway::{AuthorizationExt, AuthorizationTestExtension, DynHookContext, Gateway},
    runtime,
};
use runtime::extension::{AuthorizationDecisions, QueryElement, TokenRef};

#[derive(Default)]
pub struct MultiDirectives;

#[async_trait::async_trait]
impl AuthorizationTestExtension for MultiDirectives {
    #[allow(clippy::manual_async_fn)]
    async fn authorize_query(
        &self,
        _wasm_context: &DynHookContext,
        _headers: &tokio::sync::RwLock<http::HeaderMap>,
        _token: TokenRef<'_>,
        elements_grouped_by_directive_name: Vec<(&str, Vec<QueryElement<'_, serde_json::Value>>)>,
    ) -> Result<AuthorizationDecisions, ErrorResponse> {
        let mut element_to_error = Vec::new();
        let errors = vec![GraphqlError::new("Unauthorized", ErrorCode::Unauthorized)];
        let mut i = 0;
        for (name, elements) in elements_grouped_by_directive_name {
            match name {
                "grant" => {
                    i += elements.len() as u32;
                }
                "deny" => {
                    for _ in elements {
                        element_to_error.push((i, 0));
                        i += 1;
                    }
                }
                _ => unreachable!(),
            }
        }

        Ok(AuthorizationDecisions::DenySome {
            element_to_error,
            errors,
        })
    }
}

#[derive(Default)]
pub struct MultiDirectivesBis;

#[async_trait::async_trait]
impl AuthorizationTestExtension for MultiDirectivesBis {
    #[allow(clippy::manual_async_fn)]
    async fn authorize_query(
        &self,
        _wasm_context: &DynHookContext,
        _headers: &tokio::sync::RwLock<http::HeaderMap>,
        _token: TokenRef<'_>,
        elements_grouped_by_directive_name: Vec<(&str, Vec<QueryElement<'_, serde_json::Value>>)>,
    ) -> Result<AuthorizationDecisions, ErrorResponse> {
        let mut element_to_error = Vec::new();
        let errors = vec![GraphqlError::new("Unauthorized by bis", ErrorCode::Unauthorized)];
        let mut i = 0;
        for (name, elements) in elements_grouped_by_directive_name {
            match name {
                "grantBis" => {
                    i += elements.len() as u32;
                }
                "denyBis" => {
                    for _ in elements {
                        element_to_error.push((i, 0));
                        i += 1;
                    }
                }
                _ => unreachable!(),
            }
        }

        Ok(AuthorizationDecisions::DenySome {
            element_to_error,
            errors,
        })
    }
}

#[test]
fn multiple_directives() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@grant", "@deny"])

                type Query {
                    greeting: String @grant
                    forbidden: String @deny
                }
                "#,
                )
                .with_resolver("Query", "forbidden", serde_json::Value::String("Oh no!".to_owned()))
                .with_resolver("Query", "greeting", serde_json::Value::String("Hi!".to_owned()))
                .into_subgraph("x"),
            )
            .with_extension(AuthorizationExt::new(MultiDirectives).with_sdl(
                r#"
                directive @grant on FIELD_DEFINITION | OBJECT | INTERFACE | SCALAR | ENUM
                directive @deny on FIELD_DEFINITION | OBJECT | INTERFACE | SCALAR | ENUM
            "#,
            ))
            .build()
            .await;

        engine.post(r#"query { greeting forbidden }"#).await
    });

    insta::assert_json_snapshot!(response,  @r#"
    {
      "data": {
        "greeting": "Hi!",
        "forbidden": null
      },
      "errors": [
        {
          "message": "Unauthorized",
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
}

#[test]
fn multiple_extensions_and_directives() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@grant", "@deny"])
                extend schema @link(url: "authorizationBis-1.0.0", import: ["@grantBis", "@denyBis"])

                type Query {
                    greeting: String @grant @grantBis
                    forbidden: String @deny
                    forbiddenBis: String @denyBis
                    denied1: String @grantBis @deny
                    denied2: String @grant @denyBis
                }
                "#,
                )
                .with_resolver("Query", "forbidden", serde_json::Value::String("Oh no!".to_owned()))
                .with_resolver("Query", "greeting", serde_json::Value::String("Hi!".to_owned()))
                .into_subgraph("x"),
            )
            .with_extension(AuthorizationExt::new(MultiDirectives).with_sdl(
                r#"
                directive @grant on FIELD_DEFINITION | OBJECT | INTERFACE | SCALAR | ENUM
                directive @deny on FIELD_DEFINITION | OBJECT | INTERFACE | SCALAR | ENUM
            "#,
            ))
            .with_extension(
                AuthorizationExt::new(MultiDirectivesBis)
                    .with_name("authorizationBis")
                    .with_sdl(
                        r#"
                        directive @grantBis on FIELD_DEFINITION | OBJECT | INTERFACE | SCALAR | ENUM
                        directive @denyBis on FIELD_DEFINITION | OBJECT | INTERFACE | SCALAR | ENUM
                        "#,
                    ),
            )
            .build()
            .await;

        engine
            .post(r#"query { greeting forbidden forbiddenBis denied1 denied2 }"#)
            .await
    });

    insta::assert_json_snapshot!(response,  @r#"
    {
      "data": {
        "greeting": "Hi!",
        "forbidden": null,
        "forbiddenBis": null,
        "denied1": null,
        "denied2": null
      },
      "errors": [
        {
          "message": "Unauthorized",
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
          "message": "Unauthorized by bis",
          "locations": [
            {
              "line": 1,
              "column": 28
            }
          ],
          "path": [
            "forbiddenBis"
          ],
          "extensions": {
            "code": "UNAUTHORIZED"
          }
        }
      ]
    }
    "#);
}
