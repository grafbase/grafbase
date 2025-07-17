use std::collections::HashMap;

use engine::{ErrorCode, ErrorResponse, GraphqlError};
use graphql_mocks::FakeGithubSchema;
use integration_tests::{
    gateway::{AuthenticationExt, AuthenticationTestExtension, Gateway},
    runtime,
};
use runtime::{authentication::PublicMetadataEndpoint, extension::Token};

pub struct StaticToken(Result<Token, ErrorResponse>);

impl StaticToken {
    pub fn anonymous() -> Self {
        Self(Ok(Token::Anonymous))
    }

    pub fn bytes(bytes: impl AsRef<[u8]>) -> Self {
        Self(Ok(Token::Bytes(bytes.as_ref().to_vec())))
    }

    pub fn error_response(resp: impl Into<ErrorResponse>) -> Self {
        Self(Err(resp.into()))
    }

    pub fn claims(claims: &[(&'static str, &'static str)]) -> Self {
        let claims: HashMap<&str, &str> = claims.iter().copied().collect();
        Self(Ok(Token::Bytes(serde_json::to_vec(&claims).unwrap())))
    }
}

#[async_trait::async_trait]
impl AuthenticationTestExtension for StaticToken {
    async fn authenticate(&self, _headers: &http::HeaderMap) -> Result<Token, ErrorResponse> {
        self.0.clone()
    }

    async fn public_metadata_endpoints(&self) -> Vec<PublicMetadataEndpoint> {
        vec![]
    }
}

#[test]
fn anonymous_token() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema)
            .with_extension(AuthenticationExt::new(StaticToken::anonymous()))
            .build()
            .await;

        engine.post("query { serverVersion }").await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "serverVersion": "1"
      }
    }
    "#);
}

#[test]
fn bytes_token() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema)
            .with_extension(AuthenticationExt::new(StaticToken::bytes(Vec::new())))
            .build()
            .await;

        engine.post("query { serverVersion }").await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "serverVersion": "1"
      }
    }
    "#);
}

#[test]
fn error_response() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema)
            .with_extension(AuthenticationExt::new(StaticToken::error_response(GraphqlError::new(
                "My error message",
                ErrorCode::Unauthenticated,
            ))))
            .build()
            .await;

        engine.post("query { serverVersion }").await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "errors": [
        {
          "message": "My error message",
          "extensions": {
            "code": "UNAUTHENTICATED"
          }
        }
      ]
    }
    "#);
}
