mod error;

pub use error::VerificationError;

use jwt_compact::{
    alg::{Rsa, RsaPublicKey, StrongAlg, StrongKey},
    jwk::JsonWebKey,
    prelude::*,
    TimeOptions,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use url::Url;
use worker::kv::KvError;

const OIDC_DISCOVERY_PATH: &str = "/.well-known/openid-configuration";

// JWKS are unique with unique key IDs (kid). We could cache them for a much
// longer time, but we also need to consider that an IdP's private keys might
// get compromised. Our cache lifetime must strike a good balance between
// performance and security.
const JWKS_CACHE_TTL: u64 = 60 * 60; // 1h

#[derive(Serialize, Deserialize, Debug)]
struct OidcConfig {
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

#[derive(Serialize, Deserialize, Debug)]
struct CustomClaims {
    #[serde(rename = "iss")]
    issuer: Url,

    #[serde(rename = "sub")]
    subject: String,

    #[serde(default)]
    groups: Option<HashSet<String>>, // TODO: use configured claim name
}

#[derive(Default)]
pub struct Client {
    pub trace_id: String,
    pub http_client: surf::Client,
    pub time_opts: TimeOptions,
    pub ignore_iss_claim: bool, // used for testing
    pub jwks_cache: Option<worker::kv::KvStore>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct VerifiedToken {
    pub identity: String,
    pub groups: HashSet<String>,
}

impl Client {
    pub async fn verify_token<S: AsRef<str> + Send>(
        &self,
        token: S,
        issuer: Url,
    ) -> Result<VerifiedToken, VerificationError> {
        let token = UntrustedToken::new(&token).map_err(|_| VerificationError::InvalidToken)?;

        // We support the same signing algorithms as AppSync
        // https://docs.aws.amazon.com/appsync/latest/devguide/security-authz.html#openid-connect-authorization
        let rsa = match token.algorithm() {
            "RS256" => Rsa::rs256(),
            "RS384" => Rsa::rs384(),
            "RS512" => Rsa::rs512(),
            _ => return Err(VerificationError::UnsupportedAlgorithm),
        };

        let kid = token.header().key_id.as_ref().ok_or(VerificationError::InvalidToken)?;

        // Use JWK from cache if available
        let cached_jwk = self
            .get_jwk_from_cache(kid)
            .await
            .map_err(VerificationError::CacheError)?;

        let jwk = if let Some(cached_jwk) = cached_jwk {
            log::debug!(self.trace_id, "Found JWK {kid} in cache");
            cached_jwk
        } else {
            // Get JWKS endpoint from OIDC config
            let discovery_url = issuer.join(OIDC_DISCOVERY_PATH).expect("cannot fail");
            let oidc_config: OidcConfig = self
                .http_client
                .get(discovery_url)
                .recv_json()
                .await
                .map_err(VerificationError::HttpRequest)?;

            log::debug!(self.trace_id, "OIDC config: {oidc_config:?}");

            // XXX: we might relax this requirement and ignore issuer altogether
            if oidc_config.issuer != issuer {
                return Err(VerificationError::InvalidIssuerUrl);
            }

            // Get JWKS
            let jwks: JsonWebKeySet<'_> = self
                .http_client
                .get(oidc_config.jwks_uri)
                .recv_json()
                .await
                .map_err(VerificationError::HttpRequest)?;

            // Find JWK to verify JWT
            let jwk = jwks
                .keys
                .into_iter()
                .find(|key| &key.id == kid)
                .ok_or_else(|| VerificationError::JwkNotFound(kid.to_string()))?;

            // Add JWK to cache
            log::debug!(self.trace_id, "Adding JWK {kid} to cache");
            self.add_jwk_to_cache(&jwk)
                .await
                .map_err(VerificationError::CacheError)?;

            jwk
        };

        // Verify JWT signature
        let pub_key = RsaPublicKey::try_from(&jwk.base).map_err(|_| VerificationError::JwkFormat)?;
        let pub_key = StrongKey::try_from(pub_key).map_err(|_| VerificationError::JwkFormat)?;
        let rsa = StrongAlg(rsa);
        let token = rsa
            .validate_integrity::<CustomClaims>(&token, &pub_key)
            .map_err(VerificationError::Integrity)?;

        // Verify claims
        let claims = token.claims();

        // Check "iss" claim
        if !self.ignore_iss_claim && claims.custom.issuer != issuer {
            return Err(VerificationError::InvalidIssuerUrl);
        }

        // Check "exp" claim
        claims
            .validate_expiration(&self.time_opts)
            .map_err(VerificationError::Integrity)?;

        // Check "nbf" claim
        claims
            .validate_maturity(&self.time_opts)
            .map_err(VerificationError::Integrity)?;

        // Check "iat" claim
        // Inspired by https://github.com/jedisct1/rust-jwt-simple/blob/0.10.3/src/claims.rs#L179
        match claims.issued_at {
            Some(issued_at) if issued_at <= (self.time_opts.clock_fn)() + self.time_opts.leeway => Ok(()),
            _ => Err(VerificationError::InvalidIssueTime),
        }?;

        Ok(VerifiedToken {
            identity: claims.custom.subject.clone(),
            groups: claims.custom.groups.clone().unwrap_or_default(),
        })
    }

    async fn get_jwk_from_cache(&self, kid: &str) -> Result<Option<ExtendedJsonWebKey<'_>>, KvError> {
        if let Some(cache) = &self.jwks_cache {
            cache
                .get(kid)
                .cache_ttl(JWKS_CACHE_TTL)
                .json::<ExtendedJsonWebKey<'_>>()
                .await
        } else {
            Ok(None)
        }
    }

    async fn add_jwk_to_cache(&self, jwk: &ExtendedJsonWebKey<'_>) -> Result<(), KvError> {
        if let Some(cache) = &self.jwks_cache {
            cache
                .put(&jwk.id, &jwk)
                .expect("cannot fail")
                .expiration_ttl(JWKS_CACHE_TTL)
                .execute()
                .await
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;
    use chrono::{DateTime, Duration, NaiveDateTime, Utc};
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /* TOKEN decoded:
    {
      "header": {
        "typ": "JWT",
        "alg": "RS256",
        "kid": "ins_23i6WGIDWhlPcLeesxbmcUNLZyJ"
      },
      "payload": {
        "azp": "https://grafbase.dev",
        "exp": 1656946485,
        "iat": 1656946425,
        "iss": "https://clerk.b74v0.5y6hj.lcl.dev",
        "nbf": 1656946415,
        "sid": "sess_2BCiGPhgXZgAV00KfPrD3KSAHCO",
        "sub": "user_25sYSVDXCrWW58OusREXyl4zp30"
      }
    }
    */
    const TOKEN: &str = "eyJhbGciOiJSUzI1NiIsImtpZCI6Imluc18yM2k2V0dJRFdobFBjTGVlc3hibWNVTkxaeUoiLCJ0eXAiOiJKV1QifQ.eyJhenAiOiJodHRwczovL2dyYWZiYXNlLmRldiIsImV4cCI6MTY1Njk0NjQ4NSwiaWF0IjoxNjU2OTQ2NDI1LCJpc3MiOiJodHRwczovL2NsZXJrLmI3NHYwLjV5NmhqLmxjbC5kZXYiLCJuYmYiOjE2NTY5NDY0MTUsInNpZCI6InNlc3NfMkJDaUdQaGdYWmdBVjAwS2ZQckQzS1NBSENPIiwic3ViIjoidXNlcl8yNXNZU1ZEWENyV1c1OE91c1JFWHlsNHpwMzAifQ.CJBJD5zQIvM21YK9gSYiTjerJEyTGtwIPkG2sqicLT_GuWl7IYWGj4XPoJYLt1jYex16F5ChYapMhfYrIQq--P_0kj6DJhZ3sYrKwohRy-PFt_JJX7bsxoQG_3CdPAAPZO9WxeQnxfTYVJkAfKH2ZNGY1qvntDVZNDYEhrQIu5RKicJb0hv9gSgZSy1Q3l11mFiCS0PBiRk1QnS1xjS8aihq-Q0eQ_rWDXcoMfLbFpjLQ1LMgBDi5ihDRlCW9xouxVvW3qHWmpDW69hu2PwOIzSDByPGBsAcjwJACtZo8k2KkMkqNF1NGuhsSUZIFuNGJdtE4OVcv1VP2FIcyNqhsA";
    const TOKEN_IAT: i64 = 1_656_946_425;
    const TOKEN_SUB: &str = "user_25sYSVDXCrWW58OusREXyl4zp30";

    /* TOKEN_WITH_GROUPS decoded:
    {
      "header": {
        "typ": "JWT",
        "alg": "RS256",
        "kid": "ins_23i6WGIDWhlPcLeesxbmcUNLZyJ"
      },
      "payload": {
        "exp": 1658142514,
        "groups": [
          "admin",
          "moderator"
        ],
        "iat": 1658141914,
        "iss": "https://clerk.b74v0.5y6hj.lcl.dev",
        "jti": "ec0ffff724347261740b",
        "nbf": 1658141909,
        "sub": "user_25sYSVDXCrWW58OusREXyl4zp30"
      }
    }
        */
    const TOKEN_WITH_GROUPS: &str = "eyJhbGciOiJSUzI1NiIsImtpZCI6Imluc18yM2k2V0dJRFdobFBjTGVlc3hibWNVTkxaeUoiLCJ0eXAiOiJKV1QifQ.eyJleHAiOjE2NTgxNDI1MTQsImdyb3VwcyI6WyJhZG1pbiIsIm1vZGVyYXRvciJdLCJpYXQiOjE2NTgxNDE5MTQsImlzcyI6Imh0dHBzOi8vY2xlcmsuYjc0djAuNXk2aGoubGNsLmRldiIsImp0aSI6ImVjMGZmZmY3MjQzNDcyNjE3NDBiIiwibmJmIjoxNjU4MTQxOTA5LCJzdWIiOiJ1c2VyXzI1c1lTVkRYQ3JXVzU4T3VzUkVYeWw0enAzMCJ9.tnmYybDBENzLyGiSG4HFJQbTgOkx2MC4JyaywRksG-kDKLBnhfbJMwRULadzgAkQOFcmFJYsIYagK1VQ05HA4awy-Fq5WDSWyUWgde0SZTj12Fw6lKtlZp5FN8yRQI2h4l_zUMhG1Q0ZxPpzsxnAM5Y3TLVBmyxQeq5X8VdFbg24Ra5nFLXhTb3hTqCr6gmXQQ3kClseFgIWt-p57rv_7TSrnUe7dbSpNlqgcL1v3IquIlfGlIcS-G5jkkgKYwzclr3tYW3Eog0Vgm-HuCf-mvNCkZur3XA1SCaxJIoP0fNZK5DVsKfvSq574W1tzEV29DPN1i1j5CYmMU-sV-CmIA";
    const TOKEN_WITH_GROUPS_IAT: i64 = 1_658_141_914;

    /* TOKEN_WITH_NULL_GROUPS decoded:
    {
      "header": {
        "typ": "JWT",
        "alg": "RS256",
        "kid": "ins_23i6WGIDWhlPcLeesxbmcUNLZyJ"
      },
      "payload": {
        "exp": 1660041574,
        "groups": null,
        "iat": 1660040974,
        "iss": "https://clerk.b74v0.5y6hj.lcl.dev",
        "jti": "1c976f3586fe343c146b",
        "nbf": 1660040969,
        "sub": "user_25sYSVDXCrWW58OusREXyl4zp30"
      }
    }
    */
    const TOKEN_WITH_NULL_GROUPS: &str = "eyJhbGciOiJSUzI1NiIsImtpZCI6Imluc18yM2k2V0dJRFdobFBjTGVlc3hibWNVTkxaeUoiLCJ0eXAiOiJKV1QifQ.eyJleHAiOjE2NjAwNDE1NzQsImdyb3VwcyI6bnVsbCwiaWF0IjoxNjYwMDQwOTc0LCJpc3MiOiJodHRwczovL2NsZXJrLmI3NHYwLjV5NmhqLmxjbC5kZXYiLCJqdGkiOiIxYzk3NmYzNTg2ZmUzNDNjMTQ2YiIsIm5iZiI6MTY2MDA0MDk2OSwic3ViIjoidXNlcl8yNXNZU1ZEWENyV1c1OE91c1JFWHlsNHpwMzAifQ.vQp09Lu_z55WnrXHxC5-sy6IXSgJfjn5RnswHC8cWWDjf6xvY8x1YsSGz0IOSBOI8-_yhSyT8YJiLsGZUblPvuiD1R91Bep3ADz107t7JV0D21FgZUSsVcp-94B4vEo84lfLWynxYGf7kJ-fFgQKH9mXvZNHpcno5-xf_Ywkdjq-IhL3LnTLdpVrVuNTyWutpPL47CMfs3W71lJJ62hmLIVV3BQIDYezb9GlPXzSI4m5Rdx72lLSVjVr41rHtqdEWXAiIQ7FiKBCrMteyUoIJ12kQowEjbCGfA58L06Jk5IHBrjXnv5-ZNNnQA7pSJ6ouOHHVeBN4zhvUdhxW1mMsg";
    const TOKEN_WITH_NULL_GROUPS_IAT: i64 = 1_660_040_974;

    async fn set_up_mock_server(issuer: &Url, server: &MockServer) {
        const JWKS_PATH: &str = "/.well-known/jwks.json";
        let jwks_uri = issuer.join(JWKS_PATH).unwrap();

        Mock::given(method("GET"))
            .and(path(OIDC_DISCOVERY_PATH))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!(
                { "issuer": issuer, "jwks_uri": jwks_uri }
            )))
            .expect(1)
            .mount(server)
            .await;

        Mock::given(method("GET"))
            .and(path(JWKS_PATH))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!(
                {
                    "keys": [
                        {
                            "use": "sig",
                            "kty": "RSA",
                            "kid": "ins_23i6WGIDWhlPcLeesxbmcUNLZyJ",
                            "alg": "RS256",
                            "n": "z-Fz5w3CGNCvXJNK36DU3-t9Z6llP4j7JTJKcZWXViuqaHtnP0JuCQtesKlf58sjJinRYuSlMuRDeVZ-V7SqDqA0mfxkHqPYpgh1TOYeSMusKJjK36NlLa9nk6wPLv3C95OYTcvvEw0seE07bxiRP2U2W-ZlCE6wJQ9HtHUzLntpF5ZHLJgR3ziXTPHesp6HU4v2JfWS0laZIzgQaSXgysx6YRucZeJb0sWjPuj-aTjhXm5ThgnwzBchBIWMm2t7wh4Ma2hM_iE2MobxpOPfD25MPJ-EV-bG88B61uKbofllEn0ATs_AWSVkNvWCm9-QpTP_7MmsomrbfHEBg_VV9Q",
                            "e": "AQAB"
                        }
                    ]
                }
            )))
            .expect(1)
            .mount(server)
            .await;
    }

    #[tokio::test]
    async fn test_verify_token() {
        let server = MockServer::start().await;
        let issuer: Url = server.uri().parse().unwrap();
        set_up_mock_server(&issuer, &server).await;

        let client = {
            let leeway = Duration::seconds(5);
            let clock_fn = || DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(TOKEN_IAT, 0), Utc);
            Client {
                time_opts: TimeOptions::new(leeway, clock_fn),
                ignore_iss_claim: true,
                ..Default::default()
            }
        };

        assert_eq!(
            client.verify_token(TOKEN, issuer).await.unwrap(),
            VerifiedToken {
                identity: TOKEN_SUB.to_string(),
                groups: HashSet::new(),
            }
        );
    }

    #[tokio::test]
    async fn test_verify_token_with_null_groups() {
        let server = MockServer::start().await;
        let issuer: Url = server.uri().parse().unwrap();

        set_up_mock_server(&issuer, &server).await;

        let client = {
            let leeway = Duration::seconds(5);
            let clock_fn =
                || DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(TOKEN_WITH_NULL_GROUPS_IAT, 0), Utc);
            Client {
                time_opts: TimeOptions::new(leeway, clock_fn),
                ignore_iss_claim: true,
                ..Default::default()
            }
        };

        assert_eq!(
            client.verify_token(TOKEN_WITH_NULL_GROUPS, issuer).await.unwrap(),
            VerifiedToken {
                identity: TOKEN_SUB.to_string(),
                groups: HashSet::new(),
            }
        );
    }

    #[tokio::test]
    async fn test_verify_token_with_groups() {
        let server = MockServer::start().await;
        let issuer: Url = server.uri().parse().unwrap();
        set_up_mock_server(&issuer, &server).await;

        let client = {
            let leeway = Duration::seconds(5);
            let clock_fn = || DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(TOKEN_WITH_GROUPS_IAT, 0), Utc);
            Client {
                time_opts: TimeOptions::new(leeway, clock_fn),
                ignore_iss_claim: true,
                ..Default::default()
            }
        };

        assert_eq!(
            client.verify_token(TOKEN_WITH_GROUPS, issuer).await.unwrap(),
            VerifiedToken {
                identity: TOKEN_SUB.to_string(),
                groups: vec!["admin", "moderator"].into_iter().map(String::from).collect(),
            }
        );
    }

    #[tokio::test]
    async fn test_verify_token_from_future() {
        let server = MockServer::start().await;
        let issuer: Url = server.uri().parse().unwrap();
        set_up_mock_server(&issuer, &server).await;

        let client = {
            let leeway = Duration::seconds(5);
            // now == nbf which is 10s before the issue date.
            let clock_fn = || DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(TOKEN_IAT - 10, 0), Utc);
            Client {
                time_opts: TimeOptions::new(leeway, clock_fn),
                ignore_iss_claim: true,
                ..Default::default()
            }
        };

        assert_matches!(
            client.verify_token(TOKEN, issuer).await,
            Err(VerificationError::InvalidIssueTime)
        );
    }
}
