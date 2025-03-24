use engine::{Engine, ErrorCode, ErrorResponse, GraphqlError};
use graphql_mocks::FakeGithubSchema;
use integration_tests::{
    federation::{EngineExt, TestExtension},
    runtime,
};
use runtime::extension::Token;

use crate::federation::extensions::authentication::AuthenticationExt;

pub struct StaticToken(Result<Token, ErrorResponse>);

impl StaticToken {
    pub fn anonymous() -> Self {
        Self(Ok(Token::Anonymous))
    }

    pub fn bytes(bytes: Vec<u8>) -> Self {
        Self(Ok(Token::Bytes(bytes)))
    }

    pub fn error_response(resp: impl Into<ErrorResponse>) -> Self {
        Self(Err(resp.into()))
    }
}

#[async_trait::async_trait]
impl TestExtension for StaticToken {
    async fn authenticate(&self, _headers: &http::HeaderMap) -> Result<Token, ErrorResponse> {
        self.0.clone()
    }
}

#[test]
fn anonymous_token() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
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
        let engine = Engine::builder()
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
        let engine = Engine::builder()
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
