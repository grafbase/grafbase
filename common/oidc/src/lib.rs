mod error;

pub use error::VerificationError;

use jwt_compact::{
    alg::{Rsa, RsaPublicKey, StrongAlg, StrongKey},
    jwk::JsonWebKey,
    prelude::*,
};
use serde::{Deserialize, Serialize};
use url::Url;

const OIDC_DISCOVERY_PATH: &str = "/.well-known/openid-configuration";

#[derive(Serialize, Deserialize, Debug)]
struct OidcConfig {
    issuer: Url,
    jwks_uri: Url,
}

#[derive(Serialize, Deserialize, Debug)]
struct ExtendedJsonWebKey<'a> {
    #[serde(flatten)]
    base: JsonWebKey<'a>,
    #[serde(rename = "kid")]
    id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonWebKeySet<'a> {
    keys: Vec<ExtendedJsonWebKey<'a>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct CustomClaims {
    #[serde(rename = "iss")]
    issuer: Url,
}

pub async fn verify_token<S: AsRef<str> + Send>(token: S, issuer: Url) -> Result<(), VerificationError> {
    let token = UntrustedToken::new(&token).map_err(|_| VerificationError::InvalidToken)?;

    // TODO: AWS AppSync supports RS256, RS384, and RS512 as signing algorithms
    if token.algorithm() != "RS256" {
        return Err(VerificationError::UnsupportedAlgorithm);
    }

    let kid = token.header().key_id.as_ref().ok_or(VerificationError::InvalidToken)?;

    // Get JWKS endpoint from OIDC config
    let http_client = surf::client(); // TODO: inject client
    let discovery_url = issuer.join(OIDC_DISCOVERY_PATH).expect("cannot fail");
    let oidc_config: OidcConfig = http_client
        .get(discovery_url)
        .recv_json()
        .await
        .map_err(VerificationError::HttpRequest)?;

    if oidc_config.issuer != issuer {
        return Err(VerificationError::InvalidIssuer);
    }

    // Get JWKS
    let jwks: JsonWebKeySet<'_> = http_client
        .get(oidc_config.jwks_uri)
        .recv_json()
        .await
        .map_err(VerificationError::HttpRequest)?;

    // Find JWK to verify JWT
    let jwk = jwks
        .keys
        .iter()
        .find(|key| &key.id == kid)
        .ok_or_else(|| VerificationError::JwkNotFound(kid.to_string()))?;

    // Verify JWT signature
    let pub_key = RsaPublicKey::try_from(&jwk.base).map_err(|_| VerificationError::JwkFormat)?;
    let pub_key = StrongKey::try_from(pub_key).map_err(|_| VerificationError::JwkFormat)?;
    let rsa = StrongAlg(Rsa::rs256());
    let token = rsa
        .validate_integrity::<CustomClaims>(&token, &pub_key)
        .map_err(|_| VerificationError::InvalidToken)?;

    // TODO: verify all claims (exp, nbf, etc.)
    let claims = token.claims();

    if claims.custom.issuer != issuer {
        return Err(VerificationError::InvalidIssuer);
    }

    Ok(())
}
