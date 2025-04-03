use std::collections::HashMap;

use graphql_mocks::FakeGithubSchema;
use integration_tests::federation::GraphqlResponse;
use integration_tests::openid::{CoreClientExt, OryHydraOpenIDProvider};
use integration_tests::{
    federation::Gateway,
    openid::{AUDIENCE, JWKS_URI, OTHER_AUDIENCE},
    runtime,
};

#[test]
fn test_provider() {
    runtime().block_on(async move {
        let config = indoc::formatdoc! {r#"
            [[authentication.providers]]

            [authentication.providers.jwt]
            name = "my-jwt"

            [authentication.providers.jwt.jwks]
            url = "{JWKS_URI}"
        "#};

        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema)
            .with_toml_config(config)
            .build()
            .await;

        let token = OryHydraOpenIDProvider::default()
            .create_client()
            .await
            .get_access_token_with_client_credentials(&[])
            .await;

        let response: GraphqlResponse = engine
            .post("query { serverVersion }")
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
        let config = indoc::formatdoc! {r#"
            [[authentication.providers]]

            [authentication.providers.jwt]
            name = "my-jwt"

            [authentication.providers.jwt.jwks]
            url = "{JWKS_URI}"

            [authentication.providers.jwt.header]
            name = "X-My-JWT"
            value_prefix = "Bearer2 "
        "#};

        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema)
            .with_toml_config(config)
            .build()
            .await;

        let token = OryHydraOpenIDProvider::default()
            .create_client()
            .await
            .get_access_token_with_client_credentials(&[])
            .await;

        let response: GraphqlResponse = engine
            .post("query { serverVersion }")
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
        let config = indoc::formatdoc! {r#"
            [[authentication.providers]]

            [authentication.providers.jwt]
            name = "my-jwt"

            [authentication.providers.jwt.jwks]
            url = "{JWKS_URI}"
        "#};

        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema)
            .with_toml_config(config)
            .build()
            .await;

        // No token
        let response: GraphqlResponse = engine.post("query { serverVersion }").await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "errors": [
            {
              "message": "Unauthenticated",
              "extensions": {
                "code": "UNAUTHENTICATED"
              }
            }
          ]
        }
        "###);

        // Invalid Authorization header
        let response: GraphqlResponse = engine
            .post("query { serverVersion }")
            .header("Authorization", "something")
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "errors": [
            {
              "message": "Unauthenticated",
              "extensions": {
                "code": "UNAUTHENTICATED"
              }
            }
          ]
        }
        "###);

        // Proper HS256 JWT, but unrelated.
        let response: GraphqlResponse = engine
            .post("query { serverVersion }")
            .header("Authorization", "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c")
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "errors": [
            {
              "message": "Unauthenticated",
              "extensions": {
                "code": "UNAUTHENTICATED"
              }
            }
          ]
        }
        "###);
    });
}

#[test]
fn test_tampered_jwt() {
    runtime().block_on(async move {
        let config = indoc::formatdoc! {r#"
            [[authentication.providers]]

            [authentication.providers.jwt]
            name = "my-jwt"

            [authentication.providers.jwt.jwks]
            url = "{JWKS_URI}"
        "#};

        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema)
            .with_toml_config(config)
            .build()
            .await;

        let token = OryHydraOpenIDProvider::default()
            .create_client()
            .await
            .get_access_token_with_client_credentials(&[])
            .await;

        let token = tamper_jwt(token);

        let response: GraphqlResponse = engine
            .post("query { serverVersion }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "errors": [
            {
              "message": "Unauthenticated",
              "extensions": {
                "code": "UNAUTHENTICATED"
              }
            }
          ]
        }
        "###);
    });
}

#[allow(clippy::panic)]
pub(super) fn tamper_jwt(token: String) -> String {
    use base64::{Engine as _, engine::general_purpose};
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
        let config = indoc::formatdoc! {r#"
            [[authentication.providers]]

            [authentication.providers.jwt]
            name = "my-jwt"

            [authentication.providers.jwt.jwks]
            url = "{JWKS_URI}"
        "#};

        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema)
            .with_toml_config(config)
            .build()
            .await;

        let token = OryHydraOpenIDProvider::second_provider()
            .create_client()
            .await
            .get_access_token_with_client_credentials(&[])
            .await;

        let response: GraphqlResponse = engine
            .post("query { serverVersion }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "errors": [
            {
              "message": "Unauthenticated",
              "extensions": {
                "code": "UNAUTHENTICATED"
              }
            }
          ]
        }
        "###);
    });
}

#[test]
fn test_audience() {
    runtime().block_on(async move {
        let config = indoc::formatdoc! {r#"
            [[authentication.providers]]

            [authentication.providers.jwt]
            name = "my-jwt"

            [authentication.providers.jwt.jwks]
            url = "{JWKS_URI}"
            audience = "{AUDIENCE}"
        "#};

        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema)
            .with_toml_config(config)
            .build()
            .await;

        let token = OryHydraOpenIDProvider::default()
            .create_client()
            .await
            .get_access_token_with_client_credentials(&[("audience", AUDIENCE)])
            .await;

        let response: GraphqlResponse = engine
            .post("query { serverVersion }")
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
            .post("query { serverVersion }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "errors": [
            {
              "message": "Unauthenticated",
              "extensions": {
                "code": "UNAUTHENTICATED"
              }
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
            .post("query { serverVersion }")
            .header("Authorization", format!("Bearer {token}"))
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "errors": [
            {
              "message": "Unauthenticated",
              "extensions": {
                "code": "UNAUTHENTICATED"
              }
            }
          ]
        }
        "###);
    });
}
