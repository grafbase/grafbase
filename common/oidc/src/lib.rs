mod error;

pub use error::VerificationError;

use std::collections::HashSet;

use json_dotpath::DotPaths;
use jwt_compact::{
    alg::{Rsa, RsaPublicKey, StrongAlg, StrongKey},
    jwk::JsonWebKey,
    prelude::*,
    TimeOptions,
};
use serde::{Deserialize, Serialize};
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

    #[serde(flatten)]
    extra: serde_json::Value,
}

#[derive(Default)]
pub struct Client {
    pub trace_id: String,
    pub http_client: surf::Client,
    pub time_opts: TimeOptions, // used for testing
    pub ignore_iss_claim: bool, // used for testing
    pub groups_claim: Option<String>,
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

        // Extract groups from custom claim if present
        let groups = self
            .groups_claim
            .as_ref()
            .map(|claim| {
                claims
                    .custom
                    .extra
                    .dot_get_or_default::<HashSet<String>>(claim)
                    .map_err(|_| VerificationError::InvalidGroups(claim.to_string()))
            })
            .transpose()?
            .unwrap_or_default();

        Ok(VerifiedToken {
            identity: claims.custom.subject.clone(),
            groups,
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
                .put(&jwk.id, jwk)
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

    /* TOKEN_FROM_AUTH0 decoded:
    {
      "header": {
        "typ": "JWT",
        "alg": "RS256",
        "kid": "-PStdICfaqAFdUnDSq63E"
      },
      "payload": {
        "aud": "https://grafbase.com",
        "azp": "SvXr1yUivxX08Ajjjgxx462jJY9wqP1P",
        "exp": 1665047074,
        "gty": "client-credentials",
        "https://grafbase.com/jwt/claims/groups": [
          "admin"
        ],
        "iat": 1664960674,
        "iss": "https://gb-oidc.eu.auth0.com/",
        "sub": "SvXr1yUivxX08Ajjjgxx462jJY9wqP1P@clients"
      }
    }
    */
    const TOKEN_FROM_AUTH0: &str = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6Ii1QU3RkSUNmYXFBRmRVbkRTcTYzRSJ9.eyJodHRwczovL2dyYWZiYXNlLmNvbS9qd3QvY2xhaW1zL2dyb3VwcyI6WyJhZG1pbiJdLCJpc3MiOiJodHRwczovL2diLW9pZGMuZXUuYXV0aDAuY29tLyIsInN1YiI6IlN2WHIxeVVpdnhYMDhBampqZ3h4NDYyakpZOXdxUDFQQGNsaWVudHMiLCJhdWQiOiJodHRwczovL2dyYWZiYXNlLmNvbSIsImlhdCI6MTY2NDk2MDY3NCwiZXhwIjoxNjY1MDQ3MDc0LCJhenAiOiJTdlhyMXlVaXZ4WDA4QWpqamd4eDQ2MmpKWTl3cVAxUCIsImd0eSI6ImNsaWVudC1jcmVkZW50aWFscyJ9.HI8mxp_05-GpXHewW7_noFkUcwm0vkTf_gdmfCxh8SlNGFEZycgT_l235nfZleQ4GfsTaP0yLvpvBn5pMdHRcUnAlImvALOXAFfnYFbvwjZP0vhqfz7-vNtMdoUlOyyaxWd0idVimVPJDHmZc0lWYuUks69BdEUXyJm19XzhPodi3HtLqiF7zPOflmiOAsZjSMc5jkqVO8qv39j9WpfStr0XO97n4vGOPoA1RPenYighbethBH6tWOph2Lp7gx1HUByHQwu5GlLeDKJO-n-dAV3xAUcVKtIh_u5Yd6gofC1HTdUjWjzjrpv9SpzrqDcmzaY1WPKi-7Il17TjgXT4kA";
    const TOKEN_FROM_AUTH0_IAT: i64 = 1_664_960_674;
    const TOKEN_FROM_AUTH0_SUB: &str = "SvXr1yUivxX08Ajjjgxx462jJY9wqP1P@clients";

    /* TOKEN_WITH_NESTED_GROUPS decoded:
    {
      "header": {
        "typ": "JWT",
        "alg": "RS256",
        "kid": "ins_2DNpl5ECApCSRaSCOuwcYlirxAV"
      },
      "payload": {
        "exp": 1666715083,
        "https://grafbase.com/jwt/claims": {
          "x-grafbase-allowed-roles": [
            "editor",
            "user",
            "mod"
          ]
        },
        "iat": 1666714483,
        "iss": "https://clerk.grafbase-vercel.dev",
        "jti": "918f9036d1b5aa2a159a",
        "nbf": 1666714478,
        "sub": "user_2E4sRjokn2r14RLwhEvjVsHgCmG"
      }
    }
    */
    const TOKEN_WITH_NESTED_GROUPS: &str = "eyJhbGciOiJSUzI1NiIsImtpZCI6Imluc18yRE5wbDVFQ0FwQ1NSYVNDT3V3Y1lsaXJ4QVYiLCJ0eXAiOiJKV1QifQ.eyJleHAiOjE2NjY3MTUwODMsImh0dHBzOi8vZ3JhZmJhc2UuY29tL2p3dC9jbGFpbXMiOnsieC1ncmFmYmFzZS1hbGxvd2VkLXJvbGVzIjpbImVkaXRvciIsInVzZXIiLCJtb2QiXX0sImlhdCI6MTY2NjcxNDQ4MywiaXNzIjoiaHR0cHM6Ly9jbGVyay5ncmFmYmFzZS12ZXJjZWwuZGV2IiwianRpIjoiOTE4ZjkwMzZkMWI1YWEyYTE1OWEiLCJuYmYiOjE2NjY3MTQ0NzgsInN1YiI6InVzZXJfMkU0c1Jqb2tuMnIxNFJMd2hFdmpWc0hnQ21HIn0.jA1pmbIBn_Vkos5-irFyFhwyq4OvxnkMcs8y_joWGmGnabS9I2YM5QBP-l7ZuFY9G8b5Up_Jzr0C1IsoIr0P3fM6yGdwe8MXEvZyKRXDbScq0sUvsMJTn2FJrUL0NgE-2fOVh-H0CNqDx2c584mYDgeMGXg2po_JAhszmqqLYC8KyypF2Y_j6jtyW6kiE_nbdRLINz-lEP3Wvmy60qeZHwDX4CzcME_y7avM10vTpqSoojuaoEKdCQh7tEKIpgCI0CdDx31B_bKaHPJ3nDw8fTZQ5HxK4YXkRPIdxMjG3Dby4EKuvvegZQDoASE4gUyPJ0qBgeOXUNdf5Vk6DJX9sQ";
    const TOKEN_WITH_NESTED_GROUPS_IAT: i64 = 1_666_714_483;
    const TOKEN_WITH_NESTED_GROUPS_SUB: &str = "user_2E4sRjokn2r14RLwhEvjVsHgCmG";

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
                        // clerk.b74v0.5y6hj.lcl.dev
                        {
                            "use": "sig",
                            "kty": "RSA",
                            "kid": "ins_23i6WGIDWhlPcLeesxbmcUNLZyJ",
                            "alg": "RS256",
                            "n": "z-Fz5w3CGNCvXJNK36DU3-t9Z6llP4j7JTJKcZWXViuqaHtnP0JuCQtesKlf58sjJinRYuSlMuRDeVZ-V7SqDqA0mfxkHqPYpgh1TOYeSMusKJjK36NlLa9nk6wPLv3C95OYTcvvEw0seE07bxiRP2U2W-ZlCE6wJQ9HtHUzLntpF5ZHLJgR3ziXTPHesp6HU4v2JfWS0laZIzgQaSXgysx6YRucZeJb0sWjPuj-aTjhXm5ThgnwzBchBIWMm2t7wh4Ma2hM_iE2MobxpOPfD25MPJ-EV-bG88B61uKbofllEn0ATs_AWSVkNvWCm9-QpTP_7MmsomrbfHEBg_VV9Q",
                            "e": "AQAB"
                        },
                        // clerk.grafbase-vercel.dev
                        {
                            "use": "sig",
                            "kty": "RSA",
                            "kid": "ins_2DNpl5ECApCSRaSCOuwcYlirxAV",
                            "alg": "RS256",
                            "n": "t8IlMSSequigQ3RG1LjYyO2yY2Y1BtOLi0reYGlZ-4BYAiH99jhQQw6R7Yvg_pbgREO--34fayzx7v0te75IAGwMX22sRAJ1aZqdQxBr1lVLSjLrT-WRlIN04MucV4SK8qK8mx94fxFtMAoQxiTICxmHOzrAaoWhS64qCsekUSOiYJyVKarBBM2FDhBanbhg1l0uZnbllMK8WQ4_nLnMRzpNUaYEDJtgUOIEFrVDGEIpbMwEBl4FSDgfCNPXF-OesOPvMwWkfdCklpkj8TecKVpqYBpEodHqDlV7uHpHx8pleStLcIQn1GCqTlA1-XtU3owk2kYEBFNs-sYG-ZRNIQ",
                            "e": "AQAB"
                        },
                        // Auth0 1/2
                        {
                            "alg": "RS256",
                            "kty": "RSA",
                            "use": "sig",
                            "n": "uaJ64UOX_EBuzpCAP5KSPNT5I__wLDY6-bfUEsbImlHNtjOYUlZ48wBMc-2KO4UX1CnIHUOdE46LAOrLL8hoYKqGvJEwiumDsUtd2G8U8T1VuZgwKjjUqyhT0M-SAtXSRtyb756S9lYH3u7NHX585tsv-gJd3eDEafJQN4WrS8jFIQmi5LbmuTqc4hgNAuWGVRCYc-Sq4AxoJZnXRSH0NQOv0bGYGKXJ2Sfm-wifnm1ivEQH-JGmhn1oTrJzYGVFN8OBMYElO_hXiiWVccelpdqIrdbX3Xm9asKVin3u_GiT1CZhafu396K0JlzZX0oEoS-0yZEsCRQhYrcrmIoXzw",
                            "e": "AQAB",
                            "kid": "-PStdICfaqAFdUnDSq63E",
                            "x5t": "SFHn51pJyhkgC6H75wRnZlQdOfE",
                            "x5c": ["MIIDAzCCAeugAwIBAgIJeLAmhNIdUMvoMA0GCSqGSIb3DQEBCwUAMB8xHTAbBgNVBAMTFGdiLW9pZGMuZXUuYXV0aDAuY29tMB4XDTIyMDkyMjEwNTQ0OVoXDTM2MDUzMTEwNTQ0OVowHzEdMBsGA1UEAxMUZ2Itb2lkYy5ldS5hdXRoMC5jb20wggEiMA0GCSqGSIb3DQEBAQUAA4IBDwAwggEKAoIBAQC5onrhQ5f8QG7OkIA/kpI81Pkj//AsNjr5t9QSxsiaUc22M5hSVnjzAExz7Yo7hRfUKcgdQ50TjosA6ssvyGhgqoa8kTCK6YOxS13YbxTxPVW5mDAqONSrKFPQz5IC1dJG3JvvnpL2Vgfe7s0dfnzm2y/6Al3d4MRp8lA3hatLyMUhCaLktua5OpziGA0C5YZVEJhz5KrgDGglmddFIfQ1A6/RsZgYpcnZJ+b7CJ+ebWK8RAf4kaaGfWhOsnNgZUU3w4ExgSU7+FeKJZVxx6Wl2oit1tfdeb1qwpWKfe78aJPUJmFp+7f3orQmXNlfSgShL7TJkSwJFCFityuYihfPAgMBAAGjQjBAMA8GA1UdEwEB/wQFMAMBAf8wHQYDVR0OBBYEFFHUGRkvemN9VG0XJkns/AAjzflGMA4GA1UdDwEB/wQEAwIChDANBgkqhkiG9w0BAQsFAAOCAQEAkVqLT9/IMPGycGK//ZxxaeErHbgqujk051GJeYIJBN7kUXDrjGKo/WpiAnthw6GG5w1z9Ciw/anapRRnKauMIukhUAUrkmg0VQ0C81Jkt7dB+Jjb77z4kGmL53Ys+4ZKOHZWxRmedI4C7zHa/54rZK8oZUCgyGpM2sJ2VVkm7uXXfl93mOfqZW8PO/EVOlNrKLPC0VrrOMaynljw4NBbJfdbwsrel+VLKcZxLELyc0PeUjDYoyR56uIKNaJhu+oj2bUbU0aCYWeGp2zkSijn6WuzZbzryTHAgHxAUrKBHWbWM5Eclwa7PMP++1EYG8YwldUuw6tprZDVTAMEyjcwAA=="]
                        },
                        // Auth0 2/2
                        {
                            "alg": "RS256",
                            "kty": "RSA",
                            "use": "sig",
                            "n": "wp3UvyUh_D_cGJ7Dyu7oSnDW2xbyR1K1VX2UDmDvxEWJJWo55LWS-wCjod3r52YRJOTVwEwp_Ys39keijonfOJA3qvtMT16I8FfxhNX4P5jRV3VeqDFN4zMd23_TDxBK6pHthxB_Iaqcq_KzYzSoCsFfnOTJqV6S8uTqursZfnQXVHFdsLK4T4JArgOTLMfF1CODgOWjUYhAOu_4fAsasLN-3r9Rv5S1LEDUOZIeVBEYdCRvmZAtCldFMy0SUkD37627E1KCdRInCHjY9oYF60g3ltLqAqFj5GkNrPr8AMkTLtGf7xBe4E7l-W7tLS2uklhiOck4XPW1faIz8OiTrw",
                            "e": "AQAB",
                            "kid": "8vDmhCLv3K-68FrYZ5HUg",
                            "x5t": "2rCm4IYOopk4IIILYC0jSNNZq_s",
                            "x5c": ["MIIDAzCCAeugAwIBAgIJM6Gk08Zskw4kMA0GCSqGSIb3DQEBCwUAMB8xHTAbBgNVBAMTFGdiLW9pZGMuZXUuYXV0aDAuY29tMB4XDTIyMDkyMjEwNTQ0OVoXDTM2MDUzMTEwNTQ0OVowHzEdMBsGA1UEAxMUZ2Itb2lkYy5ldS5hdXRoMC5jb20wggEiMA0GCSqGSIb3DQEBAQUAA4IBDwAwggEKAoIBAQDCndS/JSH8P9wYnsPK7uhKcNbbFvJHUrVVfZQOYO/ERYklajnktZL7AKOh3evnZhEk5NXATCn9izf2R6KOid84kDeq+0xPXojwV/GE1fg/mNFXdV6oMU3jMx3bf9MPEErqke2HEH8hqpyr8rNjNKgKwV+c5MmpXpLy5Oq6uxl+dBdUcV2wsrhPgkCuA5Msx8XUI4OA5aNRiEA67/h8Cxqws37ev1G/lLUsQNQ5kh5UERh0JG+ZkC0KV0UzLRJSQPfvrbsTUoJ1EicIeNj2hgXrSDeW0uoCoWPkaQ2s+vwAyRMu0Z/vEF7gTuX5bu0tLa6SWGI5yThc9bV9ojPw6JOvAgMBAAGjQjBAMA8GA1UdEwEB/wQFMAMBAf8wHQYDVR0OBBYEFKoOrf8iesXeD0nJFVXoEei95HDRMA4GA1UdDwEB/wQEAwIChDANBgkqhkiG9w0BAQsFAAOCAQEAFZZqomEl/e9DXsboBmnCYFPI28ZzRyQ+J2QV7phtsBG0Vn1SVtNhY8zbvYfQdoCSHHrbdQEmG/nNKuCqh+j4uDYcKxF50QDFXCTTFTIvvlm3wSwdhWseEkyoklTQBOr4LUk7lIgllhYqGupu4ngNYjAzZ5YcGLa/q1dkTo2FKO1claIXVgrgfLmCl4hhtDdfGDPIOccQF09JOoTCag9c2Z6R5M6YP/lB+oxOq9/vGx1dPz5FHItmCv56QV5GA+UqFs8Dwln5A5bX6e3CQQXieFep71fgo0PmiGxMKNf1mERDBg4ltCSqW+OecBCRM+b2Xj5zwigUlbwt32j7iynpFg=="]
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
            let clock_fn = || DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp_opt(TOKEN_IAT, 0).unwrap(), Utc);
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
            let clock_fn = || {
                DateTime::<Utc>::from_utc(
                    NaiveDateTime::from_timestamp_opt(TOKEN_WITH_NULL_GROUPS_IAT, 0).unwrap(),
                    Utc,
                )
            };
            Client {
                time_opts: TimeOptions::new(leeway, clock_fn),
                ignore_iss_claim: true,
                groups_claim: Some("groups".to_string()),
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
            let clock_fn = || {
                DateTime::<Utc>::from_utc(
                    NaiveDateTime::from_timestamp_opt(TOKEN_WITH_GROUPS_IAT, 0).unwrap(),
                    Utc,
                )
            };
            Client {
                time_opts: TimeOptions::new(leeway, clock_fn),
                ignore_iss_claim: true,
                groups_claim: Some("groups".to_string()),
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
            let clock_fn =
                || DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp_opt(TOKEN_IAT - 10, 0).unwrap(), Utc);
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

    #[tokio::test]
    async fn test_verify_token_from_auth0() {
        let server = MockServer::start().await;
        let issuer: Url = server.uri().parse().unwrap();

        set_up_mock_server(&issuer, &server).await;

        let client = {
            let leeway = Duration::seconds(5);
            let clock_fn =
                || DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp_opt(TOKEN_FROM_AUTH0_IAT, 0).unwrap(), Utc);
            Client {
                time_opts: TimeOptions::new(leeway, clock_fn),
                ignore_iss_claim: true,
                groups_claim: Some("https://grafbase\\.com/jwt/claims/groups".to_string()),
                ..Default::default()
            }
        };

        assert_eq!(
            client.verify_token(TOKEN_FROM_AUTH0, issuer).await.unwrap(),
            VerifiedToken {
                identity: TOKEN_FROM_AUTH0_SUB.to_string(),
                groups: vec!["admin".to_string()].into_iter().collect(),
            }
        );
    }

    #[tokio::test]
    async fn test_verify_token_with_nested_groups() {
        let server = MockServer::start().await;
        let issuer: Url = server.uri().parse().unwrap();

        set_up_mock_server(&issuer, &server).await;

        let client = {
            let leeway = Duration::seconds(5);
            let clock_fn = || {
                DateTime::<Utc>::from_utc(
                    NaiveDateTime::from_timestamp_opt(TOKEN_WITH_NESTED_GROUPS_IAT, 0).unwrap(),
                    Utc,
                )
            };
            Client {
                time_opts: TimeOptions::new(leeway, clock_fn),
                ignore_iss_claim: true,
                groups_claim: Some("https://grafbase\\.com/jwt/claims.x-grafbase-allowed-roles".to_string()),
                ..Default::default()
            }
        };

        assert_eq!(
            client.verify_token(TOKEN_WITH_NESTED_GROUPS, issuer).await.unwrap(),
            VerifiedToken {
                identity: TOKEN_WITH_NESTED_GROUPS_SUB.to_string(),
                groups: vec!["editor", "user", "mod"].into_iter().map(String::from).collect(),
            }
        );
    }
}
