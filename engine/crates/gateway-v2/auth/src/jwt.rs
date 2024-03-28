use std::{borrow::Cow, collections::HashMap};

use config::v2::JwtConfig;
use futures_util::future::BoxFuture;
use jwt_compact::{jwk::JsonWebKey, Algorithm, AlgorithmExt, TimeOptions, Token, UntrustedToken};
use runtime::{auth::JwtToken, kv::KvStore};
use serde::de::DeserializeOwned;

use super::{AccessToken, Authorizer};

/// Same validation as Apollo's "JWT authentication".
pub struct JwtProvider {
    config: JwtConfig,
    kv: KvStore,
    key: String,
}

#[derive(Debug, serde::Deserialize)]
struct Jwks<'a> {
    keys: Vec<Jwk<'a>>,
}

#[derive(Debug, serde::Deserialize)]
struct Jwk<'a> {
    #[serde(flatten)]
    key: JsonWebKey<'a>,
    #[serde(rename = "kid")]
    key_id: Option<Cow<'a, str>>,
}

#[serde_with::serde_as]
#[derive(Debug, serde::Deserialize)]
struct CustomClaims {
    #[serde(default, rename = "iss")]
    issuer: Option<String>,
    #[serde_as(deserialize_as = "Option<serde_with::OneOrMany<_>>")]
    #[serde(default, rename = "aud")]
    audience: Option<Vec<String>>,
    #[serde(flatten)]
    other: HashMap<String, serde_json::Value>,
}

impl<'a> std::ops::Deref for Jwk<'a> {
    type Target = JsonWebKey<'a>;

    fn deref(&self) -> &Self::Target {
        &self.key
    }
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

    async fn load_metadata(&self) -> Option<Vec<u8>> {
        let maybe_bytes = self
            .kv
            .get(&self.key, Some(self.config.jwks.poll_interval))
            .await
            .inspect_err(|err| {
                tracing::error!("Could not load JWKS metadata from KV: {err}");
            })
            .ok()?;
        match maybe_bytes {
            Some(bytes) => Some(bytes),
            None => {
                tracing::debug!("Loading JWKS from origin");
                let bytes = async_runtime::make_send_on_wasm(async move {
                    reqwest::Client::new()
                        .get(self.config.jwks.url.clone())
                        .send()
                        .await
                        // TODO: Should be logged through the platform for customers to see those
                        // messages.
                        .inspect_err(|err| tracing::debug!("Could not fetch JWKS metadata: {err}"))?
                        .error_for_status()
                        .inspect_err(|err| tracing::debug!("Invalid response status: {err}"))?
                        .bytes()
                        .await
                        .inspect_err(|err| tracing::debug!("Could not fetch JWKS metadata: {err}"))
                })
                .await
                .ok()?;

                // No point in caching data we can't deserialize
                let _: Jwks<'_> = serde_json::from_slice(&bytes)
                    .inspect_err(|err| {
                        tracing::debug!("Could not deserialize JWKS: {err}");
                    })
                    .ok()?;

                let bytes = Vec::from(bytes);
                self.kv
                    .put(&self.key, bytes.clone(), Some(self.config.jwks.poll_interval))
                    .await
                    .inspect_err(|err| {
                        tracing::error!("Could not store JWKS metadata in KV: {err}");
                    })
                    .ok()?;
                Some(bytes)
            }
        }
    }
}

impl Authorizer for JwtProvider {
    fn get_access_token<'a>(&'a self, headers: &'a http::HeaderMap) -> BoxFuture<'a, Option<AccessToken>> {
        Box::pin(self.get_access_token(headers))
    }
}

impl JwtProvider {
    async fn get_access_token(&self, headers: &http::HeaderMap) -> Option<AccessToken> {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

        let token_str = headers
            .get(&self.config.header_name)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix(&self.config.header_value_prefix))?;

        let jwks_bytes = self.load_metadata().await?;
        let jwks: Jwks<'_> = serde_json::from_slice(&jwks_bytes)
            .inspect_err(|err| {
                tracing::debug!("Could not deserialize JWKS: {err}");
            })
            .ok()?;
        let token = decode_token(jwks.keys, UntrustedToken::new(token_str).ok()?)?;

        if let Some(expected) = self.config.jwks.issuer.as_ref() {
            if token.claims().custom.issuer.as_ref() != Some(expected) {
                return None;
            }
        }

        if let Some(expected) = self.config.jwks.audience.as_ref() {
            let audience = token.claims().custom.audience.as_ref()?;
            if audience.iter().all(|aud| aud != expected) {
                return None;
            }
        }

        let (_header, jwt_compact::Claims { custom, .. }) = token.into_parts();
        let CustomClaims {
            issuer,
            other: mut claims,
            ..
        } = custom;

        // We might want to add the rest later if asked for,
        // but 'iss' is the only one that I can think of that might be useful.
        claims.insert("iss".to_string(), issuer.into());

        let signature = URL_SAFE_NO_PAD
            .decode(token_str.rsplit('.').next().expect("valid jwt"))
            .expect("valid jwt");

        Some(AccessToken::Jwt(JwtToken { claims, signature }))
    }
}

fn decode_token(jwks: Vec<Jwk<'_>>, untrusted_token: UntrustedToken<'_>) -> Option<Token<CustomClaims>> {
    use jwt_compact::alg::*;

    let time_options = TimeOptions::default();
    jwks.iter()
        // If 'kid' was provided, we only use the jwk with the correct id.
        .filter(|jwk| match (&untrusted_token.header().key_id, &jwk.key_id) {
            (Some(expected), Some(kid)) => expected == kid,
            (Some(_), None) => false,
            (None, _) => true,
        })
        .filter_map(|jwk| match Alg::try_from(untrusted_token.algorithm()).ok()? {
            Alg::HS256 => decode(Hs256, jwk, &untrusted_token),
            Alg::HS384 => decode(Hs384, jwk, &untrusted_token),
            Alg::HS512 => decode(Hs512, jwk, &untrusted_token),
            Alg::ES256 => decode(Es256, jwk, &untrusted_token),
            Alg::RS256 => decode(Rsa::rs256(), jwk, &untrusted_token),
            Alg::RS384 => decode(Rsa::rs384(), jwk, &untrusted_token),
            Alg::RS512 => decode(Rsa::rs512(), jwk, &untrusted_token),
            Alg::PS256 => decode(Rsa::ps256(), jwk, &untrusted_token),
            Alg::PS384 => decode(Rsa::ps384(), jwk, &untrusted_token),
            Alg::PS512 => decode(Rsa::ps512(), jwk, &untrusted_token),
            Alg::EdDSA => decode(Ed25519, jwk, &untrusted_token),
        })
        .find(|token| {
            token
                .claims()
                .validate_expiration(&time_options)
                .and_then(|claims| claims.validate_maturity(&time_options))
                .is_ok()
        })
}

fn decode<A: Algorithm, T: DeserializeOwned>(
    alg: A,
    jwk: &JsonWebKey<'_>,
    untrusted_token: &UntrustedToken<'_>,
) -> Option<Token<T>>
where
    A::VerifyingKey: std::fmt::Debug + for<'a> TryFrom<&'a JsonWebKey<'a>>,
{
    let key = A::VerifyingKey::try_from(jwk).ok()?;
    alg.validator(&key)
        .validate(untrusted_token)
        .inspect_err(|err| tracing::debug!("{err:?}"))
        .ok()
}

#[derive(Debug, strum::EnumString)]
enum Alg {
    HS256,
    HS384,
    HS512,
    ES256,
    RS256,
    RS384,
    RS512,
    PS256,
    PS384,
    PS512,
    EdDSA,
}
