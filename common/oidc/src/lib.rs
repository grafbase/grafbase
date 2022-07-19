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

    #[serde(default)]
    #[serde(with = "::serde_with::rust::sets_duplicate_value_is_error")]
    groups: HashSet<String>, // TODO: use configured claim name
}

#[derive(Debug)]
pub struct VerificationOptions {
    pub issuer: Url,
    pub allowed_groups: Option<HashSet<String>>,
    pub time: Option<TimeOptions>,
    pub http_client: Option<surf::Client>,
}

pub async fn verify_token<S: AsRef<str> + Send>(token: S, opts: VerificationOptions) -> Result<(), VerificationError> {
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

    // Get JWKS endpoint from OIDC config
    let http_client = opts.http_client.unwrap_or_default();
    let discovery_url = opts.issuer.join(OIDC_DISCOVERY_PATH).expect("cannot fail");
    let oidc_config: OidcConfig = http_client
        .get(discovery_url)
        .recv_json()
        .await
        .map_err(VerificationError::HttpRequest)?;

    if oidc_config.issuer != opts.issuer {
        return Err(VerificationError::InvalidIssuerUrl);
    }

    // Get JWKS
    // TODO: cache JWKS based on kid header
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
    let rsa = StrongAlg(rsa);
    let token = rsa
        .validate_integrity::<CustomClaims>(&token, &pub_key)
        .map_err(VerificationError::Integrity)?;

    // Verify claims
    let claims = token.claims();
    let time_opts = &opts.time.unwrap_or_default();

    // Check "exp" claim
    claims
        .validate_expiration(time_opts)
        .map_err(VerificationError::Integrity)?;

    // Check "nbf" claim
    claims
        .validate_maturity(time_opts)
        .map_err(VerificationError::Integrity)?;

    // Check "iat" claim
    // Inspired by https://github.com/jedisct1/rust-jwt-simple/blob/0.10.3/src/claims.rs#L179
    match claims.issued_at {
        Some(issued_at) if issued_at <= (time_opts.clock_fn)() + time_opts.leeway => Ok(()),
        _ => Err(VerificationError::InvalidIssueTime),
    }?;

    // Check "groups" claim
    if let Some(allowed_groups) = opts.allowed_groups {
        if allowed_groups.is_disjoint(&claims.custom.groups) {
            return Err(VerificationError::InvalidGroups);
        }
    };

    Ok(())
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

        let leeway = Duration::seconds(5);
        let clock_fn = || DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(TOKEN_IAT, 0), Utc);

        let opts = VerificationOptions {
            issuer,
            allowed_groups: None,
            time: Some(TimeOptions::new(leeway, clock_fn)),
            http_client: None,
        };

        verify_token(TOKEN, opts).await.unwrap();
    }

    #[tokio::test]
    async fn should_fail_if_jwt_is_from_the_future() {
        let server = MockServer::start().await;
        let issuer: Url = server.uri().parse().unwrap();
        set_up_mock_server(&issuer, &server).await;

        let leeway = Duration::seconds(5);
        // now == nbf which is 10s before the issue date.
        let clock_fn = || DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(TOKEN_IAT - 10, 0), Utc);

        let opts = VerificationOptions {
            issuer,
            allowed_groups: None,
            time: Some(TimeOptions::new(leeway, clock_fn)),
            http_client: None,
        };

        let result = verify_token(TOKEN, opts).await;
        assert_matches!(result, Err(VerificationError::InvalidIssueTime));
    }

    #[tokio::test]
    async fn should_fail_if_jwt_lacks_groups() {
        let server = MockServer::start().await;
        let issuer: Url = server.uri().parse().unwrap();
        set_up_mock_server(&issuer, &server).await;

        let leeway = Duration::seconds(5);
        let clock_fn = || DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(TOKEN_IAT, 0), Utc);
        let opts = VerificationOptions {
            issuer: issuer.clone(),
            allowed_groups: Some(vec!["any".to_string()].into_iter().collect()),
            time: Some(TimeOptions::new(leeway, clock_fn)),
            http_client: None,
        };

        let result = verify_token(TOKEN, opts).await;
        assert_matches!(result, Err(VerificationError::InvalidGroups));
    }

    #[tokio::test]
    async fn test_verify_token_with_groups_succeeds() {
        let server = MockServer::start().await;
        let issuer: Url = server.uri().parse().unwrap();

        let valid_groups = vec![
            vec!["admin"],
            vec!["moderator"],
            vec!["admin", "moderator"],
            vec!["Admin", "moderator", "ignored"],
        ];

        for groups in valid_groups {
            set_up_mock_server(&issuer, &server).await;

            let leeway = Duration::seconds(5);
            let clock_fn = || DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(TOKEN_WITH_GROUPS_IAT, 0), Utc);
            let opts = VerificationOptions {
                issuer: issuer.clone(),
                allowed_groups: Some(groups.into_iter().map(String::from).collect()),
                time: Some(TimeOptions::new(leeway, clock_fn)),
                http_client: None,
            };
            verify_token(TOKEN_WITH_GROUPS, opts).await.unwrap();

            server.reset().await;
        }
    }

    #[tokio::test]
    async fn test_verify_token_with_groups_fails() {
        let server = MockServer::start().await;
        let issuer: Url = server.uri().parse().unwrap();

        let invalid_groups = vec![vec![], vec![""], vec!["Admin"]];

        for groups in invalid_groups {
            set_up_mock_server(&issuer, &server).await;

            let leeway = Duration::seconds(5);
            let clock_fn = || DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(TOKEN_WITH_GROUPS_IAT, 0), Utc);
            let opts = VerificationOptions {
                issuer: issuer.clone(),
                allowed_groups: Some(groups.into_iter().map(String::from).collect()),
                time: Some(TimeOptions::new(leeway, clock_fn)),
                http_client: None,
            };
            let result = verify_token(TOKEN_WITH_GROUPS, opts).await;
            assert_matches!(result, Err(VerificationError::InvalidGroups));

            server.reset().await;
        }
    }
}
