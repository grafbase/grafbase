use std::io::Write;

use elliptic_curve::pkcs8::{DecodePrivateKey, EncodePrivateKey};
use engine_v2::Engine;
use graphql_mocks::FakeGithubSchema;
use httpsig::prelude::SharedKey;
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

#[test]
fn test_message_signing_p256_jwt_happy_path() {
    // I'm hardcoding some keys here because the JWK infra in rust is a bit patchy
    // at time of writing. There's a crate, but it doesnt support everything we need and it appears
    // kind of unmaintained.
    const JWK: &str = r#"{
        "use": "sig",
        "kty": "EC",
        "kid": "4dR50dOLFlF37gbk2BKCBKdF4EhAtcQWIiRIynB3apQ",
        "crv": "P-256",
        "alg": "ES256",
        "x": "AWEDiqkKxSFrtrUWCAq7AW76G2a15_wuDTQf83STJWA",
        "y": "1d9usb-hqej5RJZnfivmO_9SgbKge3PCmZR-2rkaLgA",
        "d": "glrLr-T5EcaRfz4Im4tRU9HuTygOOdjeH3sAW1Z4SHw"
    }"#;
    const PEM: &str = indoc::indoc! {r"
        -----BEGIN PRIVATE KEY-----
        MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgglrLr+T5EcaRfz4I
        m4tRU9HuTygOOdjeH3sAW1Z4SHyhRANCAAQBYQOKqQrFIWu2tRYICrsBbvobZrXn
        /C4NNB/zdJMlYNXfbrG/oano+USWZ34r5jv/UoGyoHtzwpmUftq5Gi4A
        -----END PRIVATE KEY-----
    "};

    let key = elliptic_curve::SecretKey::from_pkcs8_pem(PEM).unwrap();
    let schema = SignedGithubSchema {
        key: graphql_mocks::VerifyingKey::Secret(httpsig::prelude::SecretKey::EcdsaP256Sha256(key.clone())),
        kid: None,
    };
    let file = KeyFile::new(JWK);
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
fn test_message_signing_p384_jwt_happy_path() {
    // I'm hardcoding some keys here because the JWK infra in rust is a bit patchy
    // at time of writing. There's a crate, but it doesnt support everything we need and it appears
    // kind of unmaintained.
    const JWK: &str = r#"{
        "use": "sig",
        "kty": "EC",
        "kid": "hEUnTRfbUOnnYt987z2yrti3LGqwsdZwd2IbHSNRqVM",
        "crv": "P-384",
        "alg": "ES384",
        "x": "WO6ufjmaqxgorIubcyf_tNISyh_YhtHUxLCqkY3HDI9qBys28XArCUew6moEpPVF",
        "y": "uhFRqXrCuFUHIXgvRzCbEv0SVkpCLOb_oIsegFIOdyOmoXp4XOdUs824aPRDXD2f",
        "d": "pwEfP2w1rkLdqVuJTVrR_gzFDt5dPgZ3aqIIdmdoCJKMKBOaqHR2VUTcV6K6ZGjX"
    }"#;
    const PEM: &str = indoc::indoc! {r"
        -----BEGIN PRIVATE KEY-----
        MIG2AgEAMBAGByqGSM49AgEGBSuBBAAiBIGeMIGbAgEBBDCnAR8/bDWuQt2pW4lN
        WtH+DMUO3l0+Bndqogh2Z2gIkowoE5qodHZVRNxXorpkaNehZANiAARY7q5+OZqr
        GCisi5tzJ/+00hLKH9iG0dTEsKqRjccMj2oHKzbxcCsJR7DqagSk9UW6EVGpesK4
        VQcheC9HMJsS/RJWSkIs5v+gix6AUg53I6ahenhc51Szzbho9ENcPZ8=
        -----END PRIVATE KEY-----
    "};

    let key = elliptic_curve::SecretKey::from_pkcs8_pem(PEM).unwrap();
    let schema = SignedGithubSchema {
        key: graphql_mocks::VerifyingKey::Secret(httpsig::prelude::SecretKey::EcdsaP384Sha384(key.clone())),
        kid: None,
    };
    let file = KeyFile::new(JWK);
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
fn test_message_signing_ed25519_jwt_happy_path() {
    // I'm hardcoding some keys here because the JWK infra in rust is a bit patchy
    // at time of writing. There's a crate, but it doesnt support everything we need and it appears
    // kind of unmaintained.
    const JWK: &str = r#"{
        "use": "sig",
        "kty": "OKP",
        "kid": "g2Jjh0V3sHLbDFHxKVYcAO4pTBYABHHj3E1CYn0Y9xQ",
        "crv": "Ed25519",
        "alg": "EdDSA",
        "x": "X8vm72PyWoWxl_mdSgR22-E56m7J7gajDI9e6S8tHWg",
        "d": "ZqlRjdgaOgOBcn1GZfxQLN3Mmgf7GY2O_bU3tAREoPc"
    }"#;
    const PEM: &str = indoc::indoc! {r"
        -----BEGIN PRIVATE KEY-----
        MC4CAQAwBQYDK2VwBCIEIGapUY3YGjoDgXJ9RmX8UCzdzJoH+xmNjv21N7QERKD3
        -----END PRIVATE KEY-----
    "};

    let key = ed25519_compact::SecretKey::from_pem(PEM).unwrap();
    let schema = SignedGithubSchema {
        key: graphql_mocks::VerifyingKey::Secret(httpsig::prelude::SecretKey::Ed25519(key)),
        kid: None,
    };
    let file = KeyFile::new(JWK);
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
fn test_message_signing_hmac_sha256_jwt_happy_path() {
    // I'm hardcoding some keys here because the JWK infra in rust is a bit patchy
    // at time of writing. There's a crate, but it doesnt support everything we need and it appears
    // kind of unmaintained.
    const JWK: &str = r#"{
        "use": "sig",
        "kty": "oct",
        "alg": "HS256",
        "k": "bFBzMG5JU0hsZm90M2VDcm84eVZHY2l4UWFJeTNnZTU"
    }"#;

    let schema = SignedGithubSchema {
        key: graphql_mocks::VerifyingKey::Shared(
            SharedKey::from_base64("bFBzMG5JU0hsZm90M2VDcm84eVZHY2l4UWFJeTNnZTU=").unwrap(),
        ),
        kid: None,
    };
    let file = KeyFile::new(JWK);
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
