use engine::ErrorResponse;
use graphql_mocks::{EchoSchema, Schema};
use integration_tests::{
    gateway::{AuthenticationExt, AuthorizationExt, AuthorizationTestExtension, Gateway},
    runtime,
};
use runtime::extension::{AuthorizationDecisions, QueryElement, TokenRef};

use crate::gateway::extensions::authentication::static_auth::StaticAuth;

#[derive(Default)]
pub struct InsertTokenAsHeader;

#[async_trait::async_trait]
impl AuthorizationTestExtension for InsertTokenAsHeader {
    #[allow(clippy::manual_async_fn)]
    async fn authorize_query(
        &self,
        _ctx: engine::EngineRequestContext,
        headers: &tokio::sync::RwLock<http::HeaderMap>,
        token: TokenRef<'_>,
        _elements_grouped_by_directive_name: Vec<(&str, Vec<QueryElement<'_, serde_json::Value>>)>,
    ) -> Result<(AuthorizationDecisions, Vec<u8>), ErrorResponse> {
        println!("{}", String::from_utf8_lossy(token.as_bytes().unwrap_or_default()));
        headers.write().await.insert(
            "token",
            http::HeaderValue::from_bytes(token.as_bytes().unwrap_or_default()).unwrap(),
        );
        Ok((AuthorizationDecisions::GrantAll, Vec::new()))
    }
}

#[test]
fn can_inject_token_into_headers() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(EchoSchema::default().with_sdl(
                r#"
                extend schema @link(url: "authorization", import: ["@auth"])

                type Query {
                    header(name: String): String @auth
                }
                "#,
            ))
            .with_extension(AuthenticationExt::new(StaticAuth::bytes("Hello world!")))
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
