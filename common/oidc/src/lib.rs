mod error;

pub use error::VerificationError;

use jwt_compact::{
    alg::{Rsa, RsaPublicKey, StrongAlg, StrongKey},
    jwk::JsonWebKey,
    prelude::*,
    TimeOptions,
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
    groups: Option<Vec<String>>, // TODO: use configured claim name
}

pub async fn verify_token<S: AsRef<str> + Send>(
    token: S,
    issuer: Url,
    groups: Option<Vec<String>>,
    time_opts: Option<TimeOptions>,
    http_client: Option<surf::Client>,
) -> Result<(), VerificationError> {
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
    let http_client = http_client.unwrap_or_default();
    let discovery_url = issuer.join(OIDC_DISCOVERY_PATH).expect("cannot fail");
    let oidc_config: OidcConfig = http_client
        .get(discovery_url)
        .recv_json()
        .await
        .map_err(VerificationError::HttpRequest)?;

    if oidc_config.issuer != issuer {
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
    let time_opts = &time_opts.unwrap_or_default();

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
    if let Some(require_groups) = groups {
        if !claims
            .custom
            .groups
            .iter()
            .flatten()
            .any(|group| require_groups.contains(group))
        {
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
    static TOKEN: &str = "eyJhbGciOiJSUzI1NiIsImtpZCI6Imluc18yM2k2V0dJRFdobFBjTGVlc3hibWNVTkxaeUoiLCJ0eXAiOiJKV1QifQ.eyJhenAiOiJodHRwczovL2dyYWZiYXNlLmRldiIsImV4cCI6MTY1Njk0NjQ4NSwiaWF0IjoxNjU2OTQ2NDI1LCJpc3MiOiJodHRwczovL2NsZXJrLmI3NHYwLjV5NmhqLmxjbC5kZXYiLCJuYmYiOjE2NTY5NDY0MTUsInNpZCI6InNlc3NfMkJDaUdQaGdYWmdBVjAwS2ZQckQzS1NBSENPIiwic3ViIjoidXNlcl8yNXNZU1ZEWENyV1c1OE91c1JFWHlsNHpwMzAifQ.CJBJD5zQIvM21YK9gSYiTjerJEyTGtwIPkG2sqicLT_GuWl7IYWGj4XPoJYLt1jYex16F5ChYapMhfYrIQq--P_0kj6DJhZ3sYrKwohRy-PFt_JJX7bsxoQG_3CdPAAPZO9WxeQnxfTYVJkAfKH2ZNGY1qvntDVZNDYEhrQIu5RKicJb0hv9gSgZSy1Q3l11mFiCS0PBiRk1QnS1xjS8aihq-Q0eQ_rWDXcoMfLbFpjLQ1LMgBDi5ihDRlCW9xouxVvW3qHWmpDW69hu2PwOIzSDByPGBsAcjwJACtZo8k2KkMkqNF1NGuhsSUZIFuNGJdtE4OVcv1VP2FIcyNqhsA";

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
        let clock_fn = || DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(1_656_946_425, 0), Utc);

        verify_token(TOKEN, issuer, None, Some(TimeOptions::new(leeway, clock_fn)), None)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn should_fail_if_jwt_is_from_the_future() {
        let server = MockServer::start().await;
        let issuer: Url = server.uri().parse().unwrap();
        set_up_mock_server(&issuer, &server).await;
        let leeway = Duration::seconds(5);
        // now == nbf which is 10s before the issue date.
        let clock_fn = || DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(1_656_946_415, 0), Utc);

        let result = verify_token(TOKEN, issuer, None, Some(TimeOptions::new(leeway, clock_fn)), None).await;
        assert_matches!(result, Err(VerificationError::InvalidIssueTime));
    }
}
