use std::io::Write;

use elliptic_curve::pkcs8::EncodePrivateKey;
use engine_v2::Engine;
use graphql_mocks::FakeGithubSchema;
use integration_tests::{federation::EngineV2Ext, runtime};

const SHARED_KEY_BASE64: &str = "aGVsbG8K";

#[test]
fn test_message_signing_shared_key_happy_path() {
    signature_success_test(
        SignedGithubSchema {
            key: shared_key(),
            kid: None,
        },
        format!(
            r#"
                [gateway.message_signatures]
                enabled = true
                key.inline = "{SHARED_KEY_BASE64}"
            "#
        ),
    );
}

#[test]
fn test_message_signing_p256_happy_path() {
    let key = elliptic_curve::SecretKey::random(&mut rand::thread_rng());
    let key_pem = key.to_pkcs8_pem(elliptic_curve::pkcs8::LineEnding::LF).unwrap();
    let schema = SignedGithubSchema {
        key: graphql_mocks::VerifyingKey::Secret(httpsig::prelude::SecretKey::EcdsaP256Sha256(key.clone())),
        kid: None,
    };
    let file = KeyFile::new(key_pem.as_str());
    let path = file.0.path().as_os_str().to_string_lossy();

    signature_success_test(
        schema,
        format!(
            r#"
                [gateway.message_signatures]
                enabled = true
                key.file = "{path}"
            "#
        ),
    )
}

#[test]
fn test_message_signing_p384_happy_path() {
    let key = elliptic_curve::SecretKey::random(&mut rand::thread_rng());
    let key_pem = key.to_pkcs8_pem(elliptic_curve::pkcs8::LineEnding::LF).unwrap();
    let schema = SignedGithubSchema {
        key: graphql_mocks::VerifyingKey::Secret(httpsig::prelude::SecretKey::EcdsaP384Sha384(key.clone())),
        kid: None,
    };
    let file = KeyFile::new(key_pem.as_str());
    let path = file.0.path().as_os_str().to_string_lossy();

    signature_success_test(
        schema,
        format!(
            r#"
                [gateway.message_signatures]
                enabled = true
                key.file = "{path}"
            "#
        ),
    )
}

#[test]
fn test_message_signing_ed25519_happy_path() {
    use ed25519_compact::*;
    let key_pair = ed25519_compact::KeyPair::from_seed(Seed::generate());

    let key_pem = key_pair.sk.to_pem();
    let schema = SignedGithubSchema {
        key: graphql_mocks::VerifyingKey::Secret(httpsig::prelude::SecretKey::Ed25519(key_pair.sk)),
        kid: None,
    };
    let file = KeyFile::new(key_pem.as_str());
    let path = file.0.path().as_os_str().to_string_lossy();

    signature_success_test(
        schema,
        format!(
            r#"
                [gateway.message_signatures]
                enabled = true
                key.file = "{path}"
            "#
        ),
    )
}

fn signature_success_test(schema: SignedGithubSchema, config: String) {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(schema)
            .with_toml_config(config)
            .build()
            .await;

        engine.post("query { serverVersion }").await
    });

    similar_asserts::assert_serde_eq!(response.body, serde_json::json!({"data": {"serverVersion": "1"}}));
}

#[test]
fn test_message_signing_with_derived_components() {
    let subgraph = SignedGithubSchema {
        key: shared_key(),
        kid: None,
    };

    let config = format!(
        r#"
            [gateway.message_signatures]
            enabled = true
            key.inline = "{SHARED_KEY_BASE64}"
            derived_components = ["method", "target_uri", "authority", "scheme", "request_target", "path"]
        "#
    );

    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(subgraph)
            .with_toml_config(config)
            .build()
            .await;

        let response = engine.post("query { serverVersion }").await;
        similar_asserts::assert_serde_eq!(response.body, serde_json::json!({"data": {"serverVersion": "1"}}));

        let request = engine
            .drain_http_requests_sent_to::<SignedGithubSchema>()
            .pop()
            .unwrap();
        let signature_input = request.headers.get("signature-input").unwrap().to_str().unwrap();

        for field in [
            "@method",
            "@target-uri",
            "@authority",
            "@scheme",
            "@request-target",
            "@path",
        ] {
            assert!(
                signature_input.contains(field),
                "could not find {field} in header: {signature_input}"
            );
        }
    });
}

#[test]
fn test_message_signing_failures() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(SignedGithubSchema {
                key: shared_key(),
                kid: None,
            })
            .build()
            .await;

        engine.post("query { serverVersion }").await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": null,
      "errors": [
        {
          "message": "Request failed with status code: 403",
          "path": [
            "serverVersion"
          ],
          "extensions": {
            "code": "SUBGRAPH_REQUEST_ERROR"
          }
        }
      ]
    }
    "###);
}

struct SignedGithubSchema {
    key: graphql_mocks::VerifyingKey,
    kid: Option<String>,
}

impl graphql_mocks::Subgraph for SignedGithubSchema {
    fn name(&self) -> String {
        FakeGithubSchema.name()
    }

    async fn start(self) -> graphql_mocks::MockGraphQlServer {
        FakeGithubSchema
            .start()
            .await
            .with_message_signing_validation(self.key, self.kid)
    }
}

fn shared_key() -> graphql_mocks::VerifyingKey {
    graphql_mocks::VerifyingKey::Shared(httpsig::prelude::SharedKey::from_base64(SHARED_KEY_BASE64).unwrap())
}

struct KeyFile(tempfile::NamedTempFile);

impl KeyFile {
    pub fn new(contents: &str) -> Self {
        let tempfile = tempfile::NamedTempFile::new().unwrap();
        let mut file = tempfile.as_file();
        file.write_all(contents.as_bytes()).unwrap();
        file.flush().unwrap();
        KeyFile(tempfile)
    }
}
