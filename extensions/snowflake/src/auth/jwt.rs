use base64::{Engine as _, engine::general_purpose};
use jwt_compact::{
    Header,
    alg::{Rsa, RsaPrivateKey},
    prelude::*,
};
use pkcs8::DecodePrivateKey as _;
use sha2::{Digest, Sha256};

/// Based on https://docs.snowflake.com/en/developer-guide/sql-api/authenticating#generating-a-jwt-in-python
pub(crate) fn generate_jwt(config: &crate::SnowflakeConfig) -> String {
    match &config.authentication {
        crate::Authentication::KeyPairJwt {
            private_key,
            public_key,
        } => {
            // Generate public key fingerprint
            let public_key_fp = generate_fingerprint(public_key);

            let account = config.account.to_uppercase();

            // User should be uppercase
            let user = config.user.to_uppercase();

            // Qualified username: ACCOUNT.USER
            let qualified_username = format!("{}.{}", account, user);

            // Create JWT header and payload
            let time_options = TimeOptions::default();

            // Expiry time is at most 1 hour (as per docs)
            // FIXME: implement refreshing JWTs
            let exp = chrono::Duration::minutes(59);

            #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
            struct CustomClaims {
                iss: String,
                sub: String,
            }

            let claims = Claims::new(CustomClaims {
                iss: format!("{}.{}", qualified_username, public_key_fp),
                sub: qualified_username,
            })
            .set_duration_and_issuance(&time_options, exp);

            let alg = Rsa::rs256();
            let private_key_der = pem::parse(private_key).expect("Private key is not valid der");
            let signing_key = RsaPrivateKey::from_pkcs8_der(private_key_der.contents()).expect("that should work too");

            alg.token(&Header::empty(), &claims, &signing_key)
                .expect("should work fine")
        }
    }
}

fn generate_fingerprint(public_key: &str) -> String {
    // Extract the DER-encoded public key
    let pem = pem::parse(public_key).expect("Failed to parse public key");
    let der_bytes = pem.contents();

    // Calculate SHA256 hash of the DER-encoded public key
    let mut hasher = Sha256::new();
    hasher.update(der_bytes);
    let hash = hasher.finalize();

    // Base64 encode the hash and format with SHA256: prefix
    format!("SHA256:{}", general_purpose::STANDARD.encode(hash))
}
