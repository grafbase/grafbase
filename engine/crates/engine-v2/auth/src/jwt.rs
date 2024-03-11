use config::v2::JwtConfig;
use futures_util::future::BoxFuture;
use jsonwebtoken::{
    jwk::{AlgorithmParameters, JwkSet},
    Algorithm, DecodingKey, TokenData,
};
use runtime::{auth::JwtToken, kv::KvStore};

use super::{AccessToken, Authorizer};

/// Same validation as Apollo's "JWT authentication".
pub struct JwtProvider {
    config: JwtConfig,
    kv: KvStore,
    key: String,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct JwtMetadata {
    jwks: JwkSet,
}

impl JwtProvider {
    pub fn new(config: JwtConfig, kv: KvStore) -> Self {
        let key: String = {
            use base64::{engine::general_purpose, Engine as _};
            use sha2::{Digest, Sha256};
            let mut key = String::from("jwks-metadata-");
            let digest = <Sha256 as Digest>::digest(config.jwks.url.to_string().as_bytes());
            key.push_str(&general_purpose::STANDARD_NO_PAD.encode(digest));
            key
        };
        JwtProvider { config, kv, key }
    }

    async fn load_metadata(&self) -> anyhow::Result<JwtMetadata> {
        let maybe_kv_metadata = self
            .kv
            .get_json_or_null(&self.key, Some(self.config.jwks.poll_interval))
            .await
            .map_err(|err| {
                tracing::error!("Could not load OpenIDConnect metadata from KV: {err}");
                err
            })?;
        let metadata = match maybe_kv_metadata {
            Some(metadata) => metadata,
            None => {
                let metadata = JwtMetadata {
                    jwks: async_runtime::make_send_on_wasm(async move {
                        reqwest::Client::new()
                            .get(self.config.jwks.url.clone())
                            .send()
                            .await
                            // TODO: Should be logged through the platform for customers to see those
                            // messages.
                            .map_err(|_| anyhow::anyhow!("Could not fetch JWKS metadata"))?
                            .json()
                            .await
                            .map_err(|_| anyhow::anyhow!("Failed to deserialize JWKS metadata"))
                    })
                    .await?,
                };
                self.kv
                    .put_json(&self.key, &metadata, Some(self.config.jwks.poll_interval))
                    .await
                    .map_err(|err| {
                        tracing::error!("Could not store JWKS metadata in KV: {err}");
                        err
                    })?;
                metadata
            }
        };
        Ok(metadata)
    }
}

impl Authorizer for JwtProvider {
    fn authorize<'a>(&'a self, headers: &'a http::HeaderMap) -> BoxFuture<'a, Option<AccessToken>> {
        Box::pin(self.get_access_token(headers))
    }
}

impl JwtProvider {
    async fn get_access_token(&self, headers: &http::HeaderMap) -> Option<AccessToken> {
        let token_str = headers
            .get(&self.config.header_name)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix(&self.config.header_value_prefix))?;

        let jose_header = jsonwebtoken::decode_header(token_str).ok()?;
        let metadata = self.load_metadata().await.ok()?;

        let validation = {
            let mut validation = jsonwebtoken::Validation::new(jose_header.alg);
            if let Some(issuer) = self.config.jwks.issuer.as_ref() {
                validation.set_issuer(&[issuer]);
            }
            if let Some(audience) = self.config.jwks.audience.as_ref() {
                validation.set_audience(&[audience]);
            } else {
                validation.validate_aud = false;
            }
            validation.validate_nbf = true;
            validation
        };

        for key in decoding_keys(&metadata.jwks, &jose_header) {
            if let Ok(TokenData { claims, .. }) = jsonwebtoken::decode(token_str, &key, &validation) {
                use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
                let signature = URL_SAFE_NO_PAD
                    .decode(token_str.rsplit('.').next().expect("valid jwt"))
                    .expect("valid jwt");
                return Some(AccessToken::Jwt(JwtToken { claims, signature }));
            }
        }

        None
    }
}

fn decoding_keys<'jwks, 'header, 'out>(
    jwks: &'jwks jsonwebtoken::jwk::JwkSet,
    header: &'header jsonwebtoken::Header,
) -> impl Iterator<Item = DecodingKey> + 'out
where
    'jwks: 'out,
    'header: 'out,
{
    jwks.keys
        .iter()
        // If 'kid' was provided, we only use the jwk with the correct id.
        .filter(|jwk| match (&header.kid, &jwk.common.key_id) {
            (Some(expected), Some(kid)) => expected == kid,
            (Some(_), None) => false,
            (None, _) => true,
        })
        // jsonwebtoken expects the appropriate DecodingKey for the algorithm
        // So we're doing a check before decoding in case 'kid' wasn't provided.
        .filter_map(move |jwk| match header.alg {
            Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => {
                if matches!(jwk.algorithm, AlgorithmParameters::OctetKey(_)) {
                    DecodingKey::from_jwk(jwk).ok()
                } else {
                    None
                }
            }
            Algorithm::ES256 | Algorithm::ES384 => {
                if matches!(jwk.algorithm, AlgorithmParameters::EllipticCurve(_)) {
                    DecodingKey::from_jwk(jwk).ok()
                } else {
                    None
                }
            }
            Algorithm::RS256
            | Algorithm::RS384
            | Algorithm::RS512
            | Algorithm::PS256
            | Algorithm::PS384
            | Algorithm::PS512 => {
                if matches!(jwk.algorithm, AlgorithmParameters::RSA(_)) {
                    DecodingKey::from_jwk(jwk).ok()
                } else {
                    None
                }
            }
            Algorithm::EdDSA => {
                if matches!(jwk.algorithm, AlgorithmParameters::OctetKeyPair(_)) {
                    DecodingKey::from_jwk(jwk).ok()
                } else {
                    None
                }
            }
        })
}
