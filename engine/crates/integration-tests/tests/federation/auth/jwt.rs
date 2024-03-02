use std::collections::HashMap;

use const_format::formatcp;
use engine_v2::Engine;
use graphql_mocks::{FakeGithubSchema, MockGraphQlServer};
use integration_tests::federation::GraphqlResponse;
use integration_tests::openid::{CoreClientExt, OryHydraOpenIDProvider};
use integration_tests::{
    federation::EngineV2Ext,
    openid::{AUDIENCE, JWKS_URI, OTHER_AUDIENCE},
    runtime,
};

const JWT_PROVIDER_CONFIG: &str = formatcp!(r#"{{ type: "jwt", jwks: {{ url: "{JWKS_URI}" }} }}"#);

#[test]
fn test_provider() {
    runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::builder()
            .with_schema("github", &github_mock)
            .await
            .with_supergraph_config(format!("extend schema @authz(providers: [{JWT_PROVIDER_CONFIG}])"))
            .finish()
            .await;

        let token = OryHydraOpenIDProvider::default()
            .create_client()
            .await
            .get_access_token_with_client_credentials(&[])
            .await;

        let response: GraphqlResponse = engine
            .execute("query { serverVersion }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "serverVersion": "1"
          }
        }
        "###);
    });
}

#[test]
fn test_different_header_location() {
    runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::builder()
            .with_schema("github", &github_mock)
            .await
            .with_supergraph_config(format!(
                r#"extend schema @authz(providers: [
                {{
                    type: "jwt",
                    jwks: {{ url: "{JWKS_URI}" }}
                    header: {{
                        name: "X-My-JWT",
                        valuePrefix: "Bearer2 "
                    }}
                }}
            ])"#
            ))
            .finish()
            .await;

        let token = OryHydraOpenIDProvider::default()
            .create_client()
            .await
            .get_access_token_with_client_credentials(&[])
            .await;

        let response: GraphqlResponse = engine
            .execute("query { serverVersion }")
            .header("X-My-JWT", format!("Bearer2 {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "serverVersion": "1"
          }
        }
        "###);
    });
}

#[test]
fn test_unauthorized() {
    runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::builder()
            .with_schema("github", &github_mock)
            .await
            .with_supergraph_config(format!("extend schema @authz(providers: [{JWT_PROVIDER_CONFIG}])"))
            .finish()
            .await;

        // No token
        let response: GraphqlResponse = engine.execute("query { serverVersion }").await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "errors": [
            {
              "message": "Unauthorized"
            }
          ]
        }
        "###);

        // Invalid Authorization header
        let response: GraphqlResponse = engine
            .execute("query { serverVersion }")
            .header("Authorization", "something")
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "errors": [
            {
              "message": "Unauthorized"
            }
          ]
        }
        "###);

        // Proper HS256 JWT, but unrelated.
        let response: GraphqlResponse = engine
            .execute("query { serverVersion }")
            .header("Authorization", "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c")
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "errors": [
            {
              "message": "Unauthorized"
            }
          ]
        }
        "###);
    });
}

#[test]
fn test_tampered_jwt() {
    runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::builder()
            .with_schema("github", &github_mock)
            .await
            .with_supergraph_config(format!("extend schema @authz(providers: [{JWT_PROVIDER_CONFIG}])"))
            .finish()
            .await;

        let token = OryHydraOpenIDProvider::default()
            .create_client()
            .await
            .get_access_token_with_client_credentials(&[])
            .await;

        let token = tamper_jwt(token);

        let response: GraphqlResponse = engine
            .execute("query { serverVersion }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "errors": [
            {
              "message": "Unauthorized"
            }
          ]
        }
        "###);
    });
}

#[allow(clippy::panic)]
pub(super) fn tamper_jwt(token: String) -> String {
    use base64::{engine::general_purpose, Engine as _};
    #[allow(clippy::panic)]
    let [header, payload, signature] = token.split('.').collect::<Vec<_>>()[..] else {
        panic!("Invalid JWT");
    };
    let mut payload = serde_json::from_slice::<HashMap<String, serde_json::Value>>(
        &general_purpose::URL_SAFE_NO_PAD.decode(payload).unwrap(),
    )
    .unwrap();
    payload.insert("sub".to_string(), serde_json::Value::String("evil admin".to_string()));
    let payload = general_purpose::URL_SAFE_NO_PAD.encode(serde_json::to_vec(&header).unwrap());
    let new_token = format!("{}.{}.{}", header, payload, signature);

    // Sanity check
    assert!(new_token != token);
    new_token
}

#[test]
fn test_wrong_provider() {
    runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::builder()
            .with_schema("github", &github_mock)
            .await
            .with_supergraph_config(format!("extend schema @authz(providers: [{JWT_PROVIDER_CONFIG}])"))
            .finish()
            .await;

        let token = OryHydraOpenIDProvider::second_provider()
            .create_client()
            .await
            .get_access_token_with_client_credentials(&[])
            .await;

        let response: GraphqlResponse = engine
            .execute("query { serverVersion }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "errors": [
            {
              "message": "Unauthorized"
            }
          ]
        }
        "###);
    });
}

#[test]
fn test_audience() {
    runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::builder()
            .with_schema("github", &github_mock)
            .await
            .with_supergraph_config(format!(
                r#"extend schema @authz(providers: [{{
                    type: "jwt",
                    jwks: {{ url: "{JWKS_URI}", audience: "{AUDIENCE}" }}
                }}])"#
            ))
            .finish()
            .await;

        let token = OryHydraOpenIDProvider::default()
            .create_client()
            .await
            .get_access_token_with_client_credentials(&[("audience", AUDIENCE)])
            .await;

        let response: GraphqlResponse = engine
            .execute("query { serverVersion }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "serverVersion": "1"
          }
        }
        "###);

        // Missing audience
        let token = OryHydraOpenIDProvider::default()
            .create_client()
            .await
            .get_access_token_with_client_credentials(&[])
            .await;

        let response: GraphqlResponse = engine
            .execute("query { serverVersion }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "errors": [
            {
              "message": "Unauthorized"
            }
          ]
        }
        "###);

        // different audience
        let token = OryHydraOpenIDProvider::default()
            .create_client()
            .await
            .get_access_token_with_client_credentials(&[("audience", OTHER_AUDIENCE)])
            .await;

        let response: GraphqlResponse = engine
            .execute("query { serverVersion }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "errors": [
            {
              "message": "Unauthorized"
            }
          ]
        }
        "###);
    });
}
