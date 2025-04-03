use engine::ErrorResponse;
use graphql_mocks::{EchoSchema, Schema};
use integration_tests::{
    gateway::{AuthenticationExt, AuthorizationExt, AuthorizationTestExtension, DynHookContext, Gateway},
    runtime,
};
use runtime::extension::{AuthorizationDecisions, QueryElement, TokenRef};

use crate::gateway::extensions::authentication::static_token::StaticToken;

#[derive(Default)]
pub struct InsertTokenAsHeader;

#[async_trait::async_trait]
impl AuthorizationTestExtension for InsertTokenAsHeader {
    #[allow(clippy::manual_async_fn)]
    async fn authorize_query(
        &self,
        _wasm_context: &DynHookContext,
        headers: &tokio::sync::RwLock<http::HeaderMap>,
        token: TokenRef<'_>,
        _elements_grouped_by_directive_name: Vec<(&str, Vec<QueryElement<'_, serde_json::Value>>)>,
    ) -> Result<AuthorizationDecisions, ErrorResponse> {
        println!("{}", String::from_utf8_lossy(token.as_bytes().unwrap_or_default()));
        headers.write().await.insert(
            "token",
            http::HeaderValue::from_bytes(token.as_bytes().unwrap_or_default()).unwrap(),
        );
        Ok(AuthorizationDecisions::GrantAll)
    }
}

#[test]
fn can_inject_token_into_headers() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(EchoSchema.with_sdl(
                r#"
                extend schema @link(url: "authorization-1.0.0", import: ["@auth"])

                type Query {
                    header(name: String): String @auth
                }
                "#,
            ))
            .with_extension(AuthenticationExt::new(StaticToken::bytes("Hello world!".into())))
            .with_extension(AuthorizationExt::new(InsertTokenAsHeader))
            .build()
            .await;

        engine.post(r#"query { header(name: "token") }"#).await
    });

    insta::assert_json_snapshot!(response,  @r#"
    {
      "data": {
        "header": "Hello world!"
      }
    }
    "#);
}
