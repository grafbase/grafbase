mod hydra;

use std::collections::HashMap;

use grafbase_sdk::test::{DynamicSchema, DynamicSubgraph, TestConfig, TestRunner};
use hydra::{CoreClientExt, OryHydraOpenIDProvider, AUDIENCE, JWKS_URI};
use indoc::formatdoc;

const CLI_PATH: &str = "../../target/debug/grafbase";
const GATEWAY_PATH: &str = "../../target/debug/grafbase-gateway";

fn config() -> String {
    formatdoc! {r#"
        [[authentication.providers]]

        [authentication.providers.extension]
        extension = "jwt"

        [authentication.providers.extension.config]
        url = "{JWKS_URI}"
    "#}
}

fn subgraph() -> DynamicSubgraph {
    DynamicSchema::builder(r#"type Query { hi: String }"#)
        .with_resolver("Query", "hi", String::from("hello"))
        .into_subgraph("test")
        .unwrap()
}

#[allow(clippy::panic)]
fn tamper_jwt(token: String) -> String {
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

#[tokio::test]
async fn without_token() {
    let config = TestConfig::builder()
        .with_cli(CLI_PATH)
        .with_gateway(GATEWAY_PATH)
        .with_subgraph(subgraph())
        .enable_networking()
        .build(config())
        .unwrap();

    let runner = TestRunner::new(config).await.unwrap();

    let result: serde_json::Value = runner.graphql_query("query { hi }").send().await.unwrap();

    insta::assert_json_snapshot!(result, @r#"
    {
      "errors": [
        {
          "message": "Unauthorized",
          "extensions": {
            "code": "UNAUTHORIZED"
          }
        }
      ]
    }
    "#);
}

#[tokio::test]
async fn with_invalid_token() {
    let config = TestConfig::builder()
        .with_cli(CLI_PATH)
        .with_gateway(GATEWAY_PATH)
        .with_subgraph(subgraph())
        .enable_networking()
        .build(config())
        .unwrap();

    let runner = TestRunner::new(config).await.unwrap();

    let result: serde_json::Value = runner
        .graphql_query("query { hi }")
        .with_header("Authorization", "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c")
        .send()
        .await
        .unwrap();

    insta::assert_json_snapshot!(result, @r#"
    {
      "errors": [
        {
          "message": "Unauthorized",
          "extensions": {
            "code": "UNAUTHORIZED"
          }
        }
      ]
    }
    "#);
}

#[tokio::test]
async fn with_valid_token() {
    let token = OryHydraOpenIDProvider::default()
        .create_client()
        .await
        .get_access_token_with_client_credentials(&[])
        .await;

    let config = TestConfig::builder()
        .with_cli(CLI_PATH)
        .with_gateway(GATEWAY_PATH)
        .with_subgraph(subgraph())
        .enable_networking()
        .build(config())
        .unwrap();

    let runner = TestRunner::new(config).await.unwrap();

    let result: serde_json::Value = runner
        .graphql_query("query { hi }")
        .with_header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    insta::assert_json_snapshot!(result, @r#"
    {
      "data": {
        "hi": "hello"
      }
    }
    "#);
}

#[tokio::test]
async fn test_different_header_location() {
    let config = formatdoc! {r#"
        [[authentication.providers]]

        [authentication.providers.extension]
        extension = "jwt"

        [authentication.providers.extension.config]
        url = "{JWKS_URI}"
        header_name = "X-My-JWT"
        header_value_prefix = "Bearer2 "
    "#};

    let config = TestConfig::builder()
        .with_cli(CLI_PATH)
        .with_gateway(GATEWAY_PATH)
        .with_subgraph(subgraph())
        .enable_networking()
        .build(config)
        .unwrap();

    let runner = TestRunner::new(config).await.unwrap();

    let token = OryHydraOpenIDProvider::default()
        .create_client()
        .await
        .get_access_token_with_client_credentials(&[])
        .await;

    let result: serde_json::Value = runner
        .graphql_query("query { hi }")
        .with_header("X-My-JWT", &format!("Bearer2 {token}"))
        .send()
        .await
        .unwrap();

    insta::assert_json_snapshot!(result, @r#"
    {
      "data": {
        "hi": "hello"
      }
    }
    "#);
}

#[tokio::test]
async fn test_tampered_jwt() {
    let config = TestConfig::builder()
        .with_cli(CLI_PATH)
        .with_gateway(GATEWAY_PATH)
        .with_subgraph(subgraph())
        .enable_networking()
        .build(config())
        .unwrap();

    let runner = TestRunner::new(config).await.unwrap();

    let token = OryHydraOpenIDProvider::default()
        .create_client()
        .await
        .get_access_token_with_client_credentials(&[])
        .await;

    let token = tamper_jwt(token);

    let result: serde_json::Value = runner
        .graphql_query("query { hi }")
        .with_header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    insta::assert_json_snapshot!(result, @r#"
    {
      "errors": [
        {
          "message": "Unauthorized",
          "extensions": {
            "code": "UNAUTHORIZED"
          }
        }
      ]
    }
    "#);
}

#[tokio::test]
async fn test_wrong_provider() {
    let config = TestConfig::builder()
        .with_cli(CLI_PATH)
        .with_gateway(GATEWAY_PATH)
        .with_subgraph(subgraph())
        .enable_networking()
        .build(config())
        .unwrap();

    let runner = TestRunner::new(config).await.unwrap();

    let token = OryHydraOpenIDProvider::second_provider()
        .create_client()
        .await
        .get_access_token_with_client_credentials(&[])
        .await;

    let result: serde_json::Value = runner
        .graphql_query("query { hi }")
        .with_header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    insta::assert_json_snapshot!(result, @r#"
    {
      "errors": [
        {
          "message": "Unauthorized",
          "extensions": {
            "code": "UNAUTHORIZED"
          }
        }
      ]
    }
    "#);
}

#[tokio::test]
async fn test_audience() {
    let config = formatdoc! {r#"
        [[authentication.providers]]

        [authentication.providers.extension]
        extension = "jwt"

        [authentication.providers.extension.config]
        url = "{JWKS_URI}"
        audience = "{AUDIENCE}"
    "#};

    let token = OryHydraOpenIDProvider::default()
        .create_client()
        .await
        .get_access_token_with_client_credentials(&[("audience", AUDIENCE)])
        .await;

    let config = TestConfig::builder()
        .with_cli(CLI_PATH)
        .with_gateway(GATEWAY_PATH)
        .with_subgraph(subgraph())
        .enable_networking()
        .build(config)
        .unwrap();

    let runner = TestRunner::new(config).await.unwrap();

    let result: serde_json::Value = runner
        .graphql_query("query { hi }")
        .with_header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    insta::assert_json_snapshot!(result, @r#"
    {
      "data": {
        "hi": "hello"
      }
    }
    "#);

    let token = OryHydraOpenIDProvider::default()
        .create_client()
        .await
        .get_access_token_with_client_credentials(&[])
        .await;

    let result: serde_json::Value = runner
        .graphql_query("query { hi }")
        .with_header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    insta::assert_json_snapshot!(result, @r#"
    {
      "errors": [
        {
          "message": "Unauthorized",
          "extensions": {
            "code": "UNAUTHORIZED"
          }
        }
      ]
    }
    "#);
}
