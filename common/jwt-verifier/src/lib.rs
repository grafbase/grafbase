use std::collections::HashSet;

use json_dotpath::DotPaths;
use jwt_compact::{jwk::JsonWebKey, prelude::*, TimeOptions};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{serde_as, OneOrMany};
use url::Url;
use worker::kv::KvError;

mod error;
#[cfg(test)]
mod tests;

pub use error::VerificationError;

const OIDC_DISCOVERY_PATH: &str = "/.well-known/openid-configuration";

// JWKS are unique with unique key IDs (kid). We could cache them for a much
// longer time, but we also need to consider that an IdP's private keys might
// get compromised. Our cache lifetime must strike a good balance between
// performance and security.
const JWKS_CACHE_TTL: u64 = 60 * 60; // 1h

#[derive(Serialize, Deserialize, Debug)]
struct OidcConfig {
    // FIXME: Issuer should be stored and handled as a string. See StringOrURI definition in https://www.rfc-editor.org/rfc/rfc7519#section-2 .
    // Converting string to Url and back alters the string representation, so for now compare `Url`-s.
    // https://linear.app/grafbase/issue/GB-3298/fix-issuer-comparison-in-oidcconfig-as-stated-by-the-fixme
    issuer: Url,
    jwks_uri: Url,
}

// A wrapper around JsonWebKey that makes the kid accessible
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

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
// FIXME: Get rid of <ISS> https://linear.app/grafbase/issue/GB-3298/fix-issuer-comparison-in-oidcconfig-as-stated-by-the-fixme
struct CustomClaims<ISS> {
    #[serde(rename = "iss")]
    issuer: ISS,

    #[serde(rename = "sub")]
    subject: Option<String>,

    // Can be either a single string or an array of strings according to
    // https://www.rfc-editor.org/rfc/rfc7519#section-4.1.3
    #[serde(rename = "aud", default)]
    #[serde_as(deserialize_as = "OneOrMany<_>")]
    audience: Vec<String>,

    #[serde(flatten)]
    extra: Value,
}

#[derive(Default)]
pub struct Client<'a> {
    pub trace_id: &'a str,
    pub http_client: reqwest::Client,
    pub time_opts: TimeOptions,        // used for testing
    pub ignore_iss_claim: bool,        // used for testing
    pub groups_claim: Option<&'a str>, // The name of the claim (json attribute) that stores groups.
    pub client_id: Option<&'a str>,    // The name of the application that must be present in the "aud" claim.
    pub jwks_cache: Option<worker::kv::KvStore>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct VerifiedToken {
    pub identity: Option<String>,
    pub groups: HashSet<String>,
}

impl<'a> Client<'a> {
    /// Verify a JSON Web Token signed with RSA + SHA (RS256, RS384, or RS512)
    /// using OIDC discovery to retrieve the public key.
    pub async fn verify_rs_token<S: AsRef<str>>(
        &self,
        token: S,
        issuer: &'a Url,
    ) -> Result<VerifiedToken, VerificationError> {
        use futures_util::TryFutureExt;
        use jwt_compact::alg::{Rsa, RsaPublicKey, StrongAlg, StrongKey};

        let token = UntrustedToken::new(&token).map_err(|_| VerificationError::InvalidToken)?;

        let rsa = match token.algorithm() {
            "RS256" => Rsa::rs256(),
            "RS384" => Rsa::rs384(),
            "RS512" => Rsa::rs512(),
            other => {
                return Err(VerificationError::UnsupportedAlgorithm {
                    algorithm: other.to_string(),
                })
            }
        };

        let kid = token.header().key_id.as_ref().ok_or(VerificationError::InvalidToken)?;

        // Use JWK from cache if available
        let cached_jwk = self
            .get_jwk_from_cache(kid, &issuer)
            .inspect_err(|err| log::error!(self.trace_id, "Cache look-up error: {err:?}"))
            .await
            .ok()
            .flatten();

        let jwk = if let Some(cached_jwk) = cached_jwk {
            log::debug!(self.trace_id, "Found JWK {kid} in cache");
            cached_jwk
        } else {
            // Get JWKS endpoint from OIDC config
            let discovery_url = issuer.join(OIDC_DISCOVERY_PATH).expect("cannot fail");
            let oidc_config: OidcConfig = self
                .http_client
                .get(discovery_url)
                .send()
                .await
                .map_err(VerificationError::HttpRequest)?
                .json()
                .await
                .map_err(VerificationError::HttpRequest)?;

            log::debug!(self.trace_id, "OIDC config: {oidc_config:?}");

            // SECURITY: This check is important to make sure that an issuer cannot
            // assume another identity
            if oidc_config.issuer != *issuer {
                return Err(VerificationError::InvalidIssuerUrl);
            }

            // Get JWKS
            let jwks: JsonWebKeySet<'_> = self
                .http_client
                .get(oidc_config.jwks_uri)
                .send()
                .await
                .map_err(VerificationError::HttpRequest)?
                .json()
                .await
                .map_err(VerificationError::HttpRequest)?;

            // Find JWK to verify JWT
            let jwk = jwks
                .keys
                .into_iter()
                .find(|key| &key.id == kid)
                .ok_or_else(|| VerificationError::JwkNotFound { kid: kid.to_string() })?;

            // Add JWK to cache
            log::debug!(self.trace_id, "Adding JWK {kid} to cache");
            let _ = self
                .add_jwk_to_cache(&jwk, issuer)
                .inspect_err(|err| log::error!(self.trace_id, "Cache write error: {err:?}"))
                .await;

            jwk
        };

        // Verify JWT signature
        let pub_key = RsaPublicKey::try_from(&jwk.base).map_err(|_| VerificationError::JwkFormat)?;
        let pub_key = StrongKey::try_from(pub_key).map_err(|_| VerificationError::JwkFormat)?;
        let rsa = StrongAlg(rsa);
        let token = rsa
            .validate_integrity::<CustomClaims<Url>>(&token, &pub_key)
            .map_err(VerificationError::Integrity)?;

        self.verify_claims(token.claims(), issuer)
    }

    /// Verify a JSON Web Token signed with HMAC + SHA (HS256, HS384, or HS512)
    /// using the provided key.
    pub fn verify_hs_token<S: AsRef<str>>(
        &self,
        token: S,
        issuer: &str,
        signing_key: &SecretString,
    ) -> Result<VerifiedToken, VerificationError> {
        use jwt_compact::alg::{Hs256, Hs256Key, Hs384, Hs384Key, Hs512, Hs512Key};
        use secrecy::ExposeSecret;

        let key = signing_key.expose_secret().as_bytes();
        let token = UntrustedToken::new(&token).map_err(|_| VerificationError::InvalidToken)?;

        let token = match token.algorithm() {
            "HS256" => Hs256
                .validate_integrity::<CustomClaims<String>>(&token, &Hs256Key::from(key))
                .map_err(VerificationError::Integrity),
            "HS384" => Hs384
                .validate_integrity::<CustomClaims<String>>(&token, &Hs384Key::from(key))
                .map_err(VerificationError::Integrity),
            "HS512" => Hs512
                .validate_integrity::<CustomClaims<String>>(&token, &Hs512Key::from(key))
                .map_err(VerificationError::Integrity),
            other => {
                return Err(VerificationError::UnsupportedAlgorithm {
                    algorithm: other.to_string(),
                })
            }
        }?;

        self.verify_claims::<String, str>(token.claims(), issuer)
    }

    fn verify_claims<ISS, C>(
        &self,
        claims: &Claims<CustomClaims<ISS>>,
        issuer: &C,
    ) -> Result<VerifiedToken, VerificationError>
    where
        C: ?Sized,
        ISS: std::borrow::Borrow<C> + PartialEq<C>,
    {
        // Check "iss" claim
        if !self.ignore_iss_claim && claims.custom.issuer != *issuer {
            return Err(VerificationError::InvalidIssuerUrl);
        }

        // Check "exp" claim
        claims
            .validate_expiration(&self.time_opts)
            .map_err(VerificationError::Integrity)?;

        // Check "nbf" claim if present
        if claims.not_before.is_some() {
            claims
                .validate_maturity(&self.time_opts)
                .map_err(VerificationError::Integrity)?;
        }

        // Check "iat" claim
        // Inspired by https://github.com/jedisct1/rust-jwt-simple/blob/0.10.3/src/claims.rs#L179
        match claims.issued_at {
            Some(issued_at) if issued_at <= (self.time_opts.clock_fn)() + self.time_opts.leeway => Ok(()),
            _ => Err(VerificationError::InvalidIssueTime),
        }?;

        // Check "aud" claim
        if let Some(client_id) = self.client_id {
            if !claims.custom.audience.contains(&client_id.to_string()) {
                return Err(VerificationError::InvalidAudience);
            };
        }

        // Extract groups from custom claim if present
        let groups = self
            .groups_claim
            .map(|claim| match claims.custom.extra.dot_get::<Value>(claim) {
                Ok(None | Some(Value::Null)) => Ok(HashSet::default()),
                Ok(Some(Value::Array(vec))) if vec == vec![Value::Null] => Ok(HashSet::default()),
                Ok(Some(Value::Array(vec))) => vec
                    .into_iter()
                    .map(|val| match val {
                        Value::String(group) => Ok(group),
                        _ => Err(VerificationError::InvalidGroups {
                            claim: (*claim).to_string(),
                        }),
                    })
                    .collect(),
                _ => Err(VerificationError::InvalidGroups {
                    claim: (*claim).to_string(),
                }),
            })
            .transpose()?
            .unwrap_or_default();

        Ok(VerifiedToken {
            identity: claims.custom.subject.clone(),
            groups,
        })
    }

    async fn get_jwk_from_cache(
        &self,
        kid: &str,
        issuer: impl std::fmt::Display,
    ) -> Result<Option<ExtendedJsonWebKey<'_>>, KvError> {
        if let Some(cache) = &self.jwks_cache {
            cache
                .get(&format!("{issuer}|{kid}"))
                .cache_ttl(JWKS_CACHE_TTL)
                .json::<ExtendedJsonWebKey<'_>>()
                .await
        } else {
            Ok(None)
        }
    }

    async fn add_jwk_to_cache(
        &self,
        jwk: &ExtendedJsonWebKey<'_>,
        issuer: impl std::fmt::Display,
    ) -> Result<(), KvError> {
        if let Some(cache) = &self.jwks_cache {
            // SECURITY: To prevent cache poisining, we not only use the kid but also the issuer
            // url. This two issuer can use the same kid without interferring with each other
            cache
                .put(&format!("{issuer}/{}", jwk.id), jwk)
                .expect("cannot fail")
                .expiration_ttl(JWKS_CACHE_TTL)
                .execute()
                .await
        } else {
            Ok(())
        }
    }
}
