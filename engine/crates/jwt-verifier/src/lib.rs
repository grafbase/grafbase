use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    time::Duration,
};

use futures_util::TryFutureExt;
use json_dotpath::DotPaths;
use jwt_compact::{
    alg::{Rsa, RsaPublicKey, StrongAlg, StrongKey},
    jwk::JsonWebKey,
    prelude::*,
    TimeOptions,
};
use log::warn;
use runtime::kv::{KvGet, KvPut, KvStore};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{serde_as, OneOrMany};
use url::Url;

mod error;
#[cfg(test)]
mod tests;

pub use error::VerificationError;

const OIDC_DISCOVERY_PATH: &str = ".well-known/openid-configuration";

// JWKS are unique with unique key IDs (kid). We could cache them for a much
// longer time, but we also need to consider that an IdP's private keys might
// get compromised. Our cache lifetime must strike a good balance between
// performance and security.
const JWKS_CACHE_TTL: u64 = 60 * 60; // 1h

#[derive(Serialize, Deserialize, Debug)]
struct OidcConfig {
    // This must be a URL, however:
    // StringOrURI values are compared as case-sensitive strings with no transformations or
    // canonicalizations applied.
    // source: https://www.rfc-editor.org/rfc/rfc7519#section-2
    issuer: String,
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
#[derive(Serialize, Deserialize, Debug, Clone)]
struct CustomClaims {
    // Optional as per https://www.rfc-editor.org/rfc/rfc7519#section-4.1.1 .
    // Mandatory in case of OIDC discovery: https://openid.net/specs/openid-connect-discovery-1_0.html#ProviderMetadata
    #[serde(rename = "iss", skip_serializing_if = "Option::is_none")]
    issuer: Option<String>,

    #[serde(rename = "sub", skip_serializing_if = "Option::is_none")]
    subject: Option<String>,

    // Can be either a single string or an array of strings according to
    // https://www.rfc-editor.org/rfc/rfc7519#section-4.1.3
    #[serde(rename = "aud", default, skip_serializing_if = "Vec::is_empty")]
    #[serde_as(deserialize_as = "OneOrMany<_>", serialize_as = "OneOrMany<_>")]
    audience: Vec<String>,

    #[serde(flatten)]
    extra: Value,
}

#[derive(Default)]
pub struct Client<'a, Kv> {
    pub trace_id: &'a str,
    pub http_client: reqwest::Client,
    pub time_opts: TimeOptions,        // used for testing
    pub groups_claim: Option<&'a str>, // The name of the claim (json attribute) that stores groups.
    pub client_id: Option<&'a str>,    // The name of the application that must be present in the "aud" claim.
    pub jwks_cache: Option<Kv>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct VerifiedToken {
    pub identity: Option<String>,
    pub groups: BTreeSet<String>,
    pub token_claims: BTreeMap<String, Value>,
}

impl<'a, Kv: KvStore> Client<'a, Kv> {
    fn joinable_url(&self, url: &url::Url) -> url::Url {
        if url.to_string().ends_with('/') {
            url.clone()
        } else {
            log::debug!(self.trace_id, "Appending trailing slash to {url}");
            format!("{url}/").parse().expect("url was already parsed")
        }
    }

    /// Verify a JSON Web Token signed with RSA + SHA (RS256, RS384, or RS512)
    /// using OIDC discovery to retrieve the public key.
    pub async fn verify_rs_token_using_oidc_discovery<S: AsRef<str>>(
        &self,
        token: S,
        issuer_base_url: &url::Url,
        expected_issuer: &'a str,
    ) -> Result<VerifiedToken, VerificationError> {
        let token = UntrustedToken::new(&token).map_err(|err| {
            log::warn!(self.trace_id, "Cannot parse JWT - {err}");
            VerificationError::InvalidToken
        })?;
        log::trace!(
            self.trace_id,
            "Untrusted token algorith {}, header: {:?}",
            token.algorithm(),
            token.header()
        );

        let rsa = Self::get_rsa_algorithm(&token)?;

        let Some(kid) = token.header().key_id.as_ref() else {
            log::warn!(self.trace_id, "Rejecting JWT, kid is not present");
            return Err(VerificationError::InvalidToken);
        };

        // Use JWK from cache if available
        let discovery_url = self
            .joinable_url(issuer_base_url)
            .join(OIDC_DISCOVERY_PATH)
            .expect("cannot fail");
        let caching_key = CachingKey::Oidc {
            discovery_url: &discovery_url,
            kid,
        };

        let cached_jwk = self
            .get_jwk_from_cache(&caching_key)
            .inspect_err(|err| log::error!(self.trace_id, "Cache look-up error: {err:?}"))
            .await
            .ok()
            .flatten();

        let jwk = if let Some(cached_jwk) = cached_jwk {
            log::debug!(self.trace_id, "Found JWK {kid} in cache");
            cached_jwk
        } else {
            log::trace!(self.trace_id, "Getting oidc config from {discovery_url:?}");
            // Get JWKS endpoint from OIDC config
            let oidc_config: OidcConfig = self
                .http_client
                .get(discovery_url.clone())
                .send()
                .await
                .map_err(VerificationError::HttpRequest)?
                .json()
                .await
                .map_err(VerificationError::HttpRequest)?;

            log::debug!(self.trace_id, "OIDC config: {oidc_config:?}");

            // SECURITY: This check is important to make sure that an issuer cannot
            // assume another identity
            self.verify_issuer((expected_issuer, Some(issuer_base_url)), (&oidc_config.issuer, None))?;

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

            let _ = self
                .add_jwk_to_cache(&caching_key, &jwk)
                .inspect_err(|err| log::error!(self.trace_id, "Cache write error: {err:?}"))
                .await;

            jwk
        };

        // Verify JWT signature
        let pub_key = RsaPublicKey::try_from(&jwk.base).map_err(|_| VerificationError::JwkFormat)?;
        let pub_key = StrongKey::try_from(pub_key).map_err(|_| VerificationError::JwkFormat)?;
        let rsa = StrongAlg(rsa);
        let token = rsa
            .validator(&pub_key)
            .validate(&token)
            .map_err(VerificationError::Integrity)?;

        self.verify_claims(token.claims(), Some(expected_issuer))
    }

    /// Verify a JSON Web Token signed with RSA + SHA (RS256, RS384, or RS512)
    /// using JWKS endpoint to retrieve the public key.
    pub async fn verify_rs_token_using_jwks_endpoint<S: AsRef<str>>(
        &self,
        token: S,
        jwks_endpoint_url: &'a Url,
        expected_issuer: Option<&'a str>,
    ) -> Result<VerifiedToken, VerificationError> {
        let token = UntrustedToken::new(&token).map_err(|err| {
            log::warn!(self.trace_id, "Cannot parse JWT - {err}");
            VerificationError::InvalidToken
        })?;
        log::trace!(
            self.trace_id,
            "Untrusted token algorithm {}, header: {:?}",
            token.algorithm(),
            token.header()
        );

        let rsa = Self::get_rsa_algorithm(&token)?;

        let kid = token.header().key_id.as_ref().ok_or({
            log::warn!(self.trace_id, "Rejecting JWT, kid is not present");
            VerificationError::InvalidToken
        })?;

        let caching_key = CachingKey::Jwks { jwks_endpoint_url, kid };
        let cached_jwk = self
            .get_jwk_from_cache(&caching_key)
            .inspect_err(|err| log::error!(self.trace_id, "Cache look-up error: {err:?}"))
            .await
            .ok()
            .flatten();
        let jwk = if let Some(cached_jwk) = cached_jwk {
            log::debug!(self.trace_id, "Found JWK {kid} in cache");
            cached_jwk
        } else {
            let jwk = {
                // Get JWKS
                let jwks: JsonWebKeySet<'_> = self
                    .http_client
                    .get(jwks_endpoint_url.clone())
                    .send()
                    .await
                    .map_err(VerificationError::HttpRequest)?
                    .json()
                    .await
                    .map_err(VerificationError::HttpRequest)?;

                // Find JWK to verify JWT
                jwks.keys
                    .into_iter()
                    .find(|key| &key.id == kid)
                    .ok_or_else(|| VerificationError::JwkNotFound { kid: kid.to_string() })?
            };

            let _ = self
                .add_jwk_to_cache(&caching_key, &jwk)
                .inspect_err(|err| log::error!(self.trace_id, "Cache write error: {err:?}"))
                .await;
            jwk
        };

        // Verify JWT signature
        let pub_key = RsaPublicKey::try_from(&jwk.base).map_err(|_| VerificationError::JwkFormat)?;
        let pub_key = StrongKey::try_from(pub_key).map_err(|_| VerificationError::JwkFormat)?;
        let rsa = StrongAlg(rsa);
        let token = rsa
            .validator(&pub_key)
            .validate(&token)
            .map_err(VerificationError::Integrity)?;

        self.verify_claims(token.claims(), expected_issuer)
    }

    /// Verify a JSON Web Token signed with HMAC + SHA (HS256, HS384, or HS512)
    /// using the provided key.
    pub fn verify_hs_token<S: AsRef<str>>(
        &self,
        token: S,
        expected_issuer: &str,
        signing_key: &SecretString,
    ) -> Result<VerifiedToken, VerificationError> {
        use jwt_compact::alg::{Hs256, Hs256Key, Hs384, Hs384Key, Hs512, Hs512Key};
        use secrecy::ExposeSecret;

        let key = signing_key.expose_secret().as_bytes();
        let token = UntrustedToken::new(&token).map_err(|err| {
            log::warn!(self.trace_id, "Cannot parse JWT - {err}");
            VerificationError::InvalidToken
        })?;
        log::trace!(
            self.trace_id,
            "Untrusted token algorithm {}, header: {:?}",
            token.algorithm(),
            token.header()
        );

        let token = match token.algorithm() {
            "HS256" => Hs256
                .validator(&Hs256Key::from(key))
                .validate(&token)
                .map_err(VerificationError::Integrity),
            "HS384" => Hs384
                .validator(&Hs384Key::from(key))
                .validate(&token)
                .map_err(VerificationError::Integrity),
            "HS512" => Hs512
                .validator(&Hs512Key::from(key))
                .validate(&token)
                .map_err(VerificationError::Integrity),
            other => {
                return Err(VerificationError::UnsupportedAlgorithm {
                    algorithm: other.to_string(),
                })
            }
        }?;

        self.verify_claims(token.claims(), Some(expected_issuer))
    }

    fn get_rsa_algorithm(token: &UntrustedToken<'_>) -> Result<Rsa, VerificationError> {
        match token.algorithm() {
            "RS256" => Ok(Rsa::rs256()),
            "RS384" => Ok(Rsa::rs384()),
            "RS512" => Ok(Rsa::rs512()),
            other => Err(VerificationError::UnsupportedAlgorithm {
                algorithm: other.to_string(),
            }),
        }
    }

    fn verify_issuer(
        &self,
        expected: (&str, Option<&url::Url>),
        actual: (&str, Option<&url::Url>),
    ) -> Result<(), VerificationError> {
        if expected.0 == actual.0 {
            Ok(())
        } else {
            // Backwards compatibility: Previously the issuer was first parsed as URL and then compared which is against the spec:
            // https://www.rfc-editor.org/rfc/rfc7519#section-4.1.1
            // Attempt to convert both sides to URLs and compare them.
            // This may add a trailing slash if the URL does not have a path section, and therefore should be safe.
            let expected_url = expected.1.map(Cow::Borrowed).map(Ok).unwrap_or_else(|| {
                expected
                    .0
                    .parse::<url::Url>()
                    .map(Cow::Owned)
                    .map_err(VerificationError::IssuerFormat)
            })?;
            let actual_url = actual.1.map(|url| Ok(Cow::Borrowed(url))).unwrap_or_else(|| {
                actual
                    .0
                    .parse::<url::Url>()
                    .map(Cow::Owned)
                    .map_err(VerificationError::IssuerFormat)
            })?;
            if expected_url == actual_url {
                log::debug!(
                    self.trace_id,
                    "Passing issuer verification although expected '{}' does not match exactly the actual '{}'",
                    expected.0,
                    actual.0
                );
                Ok(())
            } else {
                warn!(
                    self.trace_id,
                    "Actual issuer {} does not match the expected {}", actual.0, expected.0
                );
                Err(VerificationError::IssuerClaimMismatch)
            }
        }
    }

    fn verify_claims(
        &self,
        claims: &Claims<CustomClaims>,
        expected_issuer: Option<&str>,
    ) -> Result<VerifiedToken, VerificationError> {
        // Check "iss" claim if expected_issuer is set.
        if let Some(expected_issuer) = expected_issuer {
            let Some(actual_issuer) = &claims.custom.issuer else {
                log::warn!(self.trace_id, "JWT does not contain the 'iss' claim that is required");
                return Err(VerificationError::IssuerClaimMismatch);
            };
            self.verify_issuer((expected_issuer, None), (actual_issuer, None))?;
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
                Ok(None | Some(Value::Null)) => Ok(BTreeSet::default()),
                Ok(Some(Value::Array(vec))) if vec == vec![Value::Null] => Ok(BTreeSet::default()),
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
            token_claims: serde_json::from_value(
                serde_json::to_value(&claims.custom).expect("custom claims should be serializable"),
            )
            .expect("should be deserializable to map"),
        })
    }

    async fn get_jwk_from_cache(
        &self,
        caching_key: &CachingKey<'_>,
    ) -> Result<Option<ExtendedJsonWebKey<'_>>, Kv::Error> {
        if let Some(cache) = &self.jwks_cache {
            cache
                .get(&caching_key.key())
                .cache_ttl(Duration::from_secs(JWKS_CACHE_TTL))
                .json::<ExtendedJsonWebKey<'_>>()
                .await
        } else {
            Ok(None)
        }
    }

    async fn add_jwk_to_cache(
        &self,
        caching_key: &CachingKey<'_>,
        jwk: &ExtendedJsonWebKey<'_>,
    ) -> Result<(), Kv::Error> {
        assert_eq!(caching_key.kid(), jwk.id, "key identifier must be the same");
        if let Some(cache) = &self.jwks_cache {
            let key = caching_key.key();
            log::debug!(self.trace_id, "Adding {key} to cache");
            cache
                .put(&key, jwk)?
                .expiration_ttl(Duration::from_secs(JWKS_CACHE_TTL))
                .execute()
                .await
        } else {
            Ok(())
        }
    }
}

const CACHING_SEPARATOR: &str = "|";

enum CachingKey<'a> {
    Oidc { discovery_url: &'a Url, kid: &'a str },
    Jwks { jwks_endpoint_url: &'a Url, kid: &'a str },
}

impl<'a> CachingKey<'a> {
    // SECURITY: The key identifier (kid) is an opaque string that does not have to be derived from its public key.
    // To prevent cache poisining (malicious issuer reusing the same kid),
    // derive the cache key from provider type + url + kid.
    fn key(&self) -> String {
        match self {
            Self::Oidc { discovery_url, kid } => {
                format!("OIDC{CACHING_SEPARATOR}{discovery_url}{CACHING_SEPARATOR}{kid}",)
            }
            Self::Jwks { jwks_endpoint_url, kid } => {
                format!("JWKS{CACHING_SEPARATOR}{jwks_endpoint_url}{CACHING_SEPARATOR}{kid}",)
            }
        }
    }

    fn kid(&self) -> &str {
        match self {
            Self::Oidc { kid, .. } | Self::Jwks { kid, .. } => kid,
        }
    }
}
