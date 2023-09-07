use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use serde_json::json;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

use super::*;

const JWKS_PATH: &str = ".well-known/jwks.json";

async fn set_up_oidc_server(issuer: &Url, server: &MockServer) {
    assert!(issuer.path().ends_with('/'));
    let discovery_url = issuer.join(OIDC_DISCOVERY_PATH).unwrap();
    let jwks_uri = issuer.join(JWKS_PATH).unwrap();
    Mock::given(method("GET"))
        .and(path(discovery_url.path()))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!(
            { "issuer": issuer, "jwks_uri": jwks_uri }
        )))
        .expect(1)
        .mount(server)
        .await;

    set_up_jwks_server(jwks_uri.path(), server).await;
}

async fn set_up_jwks_server(jwks_path: &str, server: &MockServer) {
    Mock::given(method("GET"))
  .and(path(jwks_path))
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
              },
              // Hanko instance https://b91ba64e-b8fb-4262-92ff-211e89810456.hanko.io
              {
                  "alg": "RS256",
                  "kty": "RSA",
                  "use": "sig",
                  "kid": "71f19172-229c-4de5-9b08-c533fd2cee8a",
                  "e": "AQAB",
                  "n": "yDIIV8slg_uObNLOAuIAe1tchGshxNy2JNWSHQZ22chk39nKT5bUvUeTNRuc9Gv7_A4vzC8GP0HZZUv3roxsycbXmJdplsj8IPtgRZG6E4z3yg_GYc0R91BYjREvQ2Bkl90t9AyeY-dvahLEctW3ZQbw4GpkuwsKOReV2L8zahFf9CsZH8E_uxWWgS7a_ptuYYYX8SywCp-WkMAfsFaFb4FPysv3zSSd-J1czpsUqQdiaf2WXs8WH21WQCIGMW2sxUDce12OtoEAF7ALIrdLrDhQlv9WTE7paoID0yGbv4Ozl4a-fxD40fQxkkdt8PFiqlDS7oxpk66E7DZDCvt7-Irca4psySqESGw2s4QxjIA_BkJth7sJsIs2M7S_bAIE1J2DplXADU5UoWM4NnsV7ehD-LMKqoWMP5LgrTvEbA-cqiBS5IJPgOQ0lRwgqmAF4K_xjB8juLWId3q0ge-Gk70Tr7SHI6x_lRrsPvM9kOJz_hTRw_3tILWIMq2aPglmTXAlqoIcn07us77BVKs5c5cRWAWfqyLye-a6RfMwABB1BaTm0Mth6ZH5AWugQQcdcybadq6Unejkf-T9l0teZ_W5M8dgMhXt0dNopKvmPGWvK5Pu6a2Oi_KLS-gTqCt69sf5w6CdG4-id7Pt9xBti172W_Lk6gmVdMgRWbI4Jjs"
              },
              // Azure AD endpoint from Kiiwa
            {
                "kty": "RSA",
                "use": "sig",
                "kid": "-KI3Q9nNR7bRofxmeZoXqbHZGew",
                "x5t": "-KI3Q9nNR7bRofxmeZoXqbHZGew",
                "n": "tJL6Wr2JUsxLyNezPQh1J6zn6wSoDAhgRYSDkaMuEHy75VikiB8wg25WuR96gdMpookdlRvh7SnRvtjQN9b5m4zJCMpSRcJ5DuXl4mcd7Cg3Zp1C5-JmMq8J7m7OS9HpUQbA1yhtCHqP7XA4UnQI28J-TnGiAa3viPLlq0663Cq6hQw7jYo5yNjdJcV5-FS-xNV7UHR4zAMRruMUHxte1IZJzbJmxjKoEjJwDTtcd6DkI3yrkmYt8GdQmu0YBHTJSZiz-M10CY3LbvLzf-tbBNKQ_gfnGGKF7MvRCmPA_YF_APynrIG7p4vPDRXhpG3_CIt317NyvGoIwiv0At83kQ",
                "e": "AQAB",
                "x5c": [
                    "MIIDBTCCAe2gAwIBAgIQGQ6YG6NleJxJGDRAwAd/ZTANBgkqhkiG9w0BAQsFADAtMSswKQYDVQQDEyJhY2NvdW50cy5hY2Nlc3Njb250cm9sLndpbmRvd3MubmV0MB4XDTIyMTAwMjE4MDY0OVoXDTI3MTAwMjE4MDY0OVowLTErMCkGA1UEAxMiYWNjb3VudHMuYWNjZXNzY29udHJvbC53aW5kb3dzLm5ldDCCASIwDQYJKoZIhvcNAQEBBQADggEPADCCAQoCggEBALSS+lq9iVLMS8jXsz0IdSes5+sEqAwIYEWEg5GjLhB8u+VYpIgfMINuVrkfeoHTKaKJHZUb4e0p0b7Y0DfW+ZuMyQjKUkXCeQ7l5eJnHewoN2adQufiZjKvCe5uzkvR6VEGwNcobQh6j+1wOFJ0CNvCfk5xogGt74jy5atOutwquoUMO42KOcjY3SXFefhUvsTVe1B0eMwDEa7jFB8bXtSGSc2yZsYyqBIycA07XHeg5CN8q5JmLfBnUJrtGAR0yUmYs/jNdAmNy27y83/rWwTSkP4H5xhihezL0QpjwP2BfwD8p6yBu6eLzw0V4aRt/wiLd9ezcrxqCMIr9ALfN5ECAwEAAaMhMB8wHQYDVR0OBBYEFJcSH+6Eaqucndn9DDu7Pym7OA8rMA0GCSqGSIb3DQEBCwUAA4IBAQADKkY0PIyslgWGmRDKpp/5PqzzM9+TNDhXzk6pw8aESWoLPJo90RgTJVf8uIj3YSic89m4ftZdmGFXwHcFC91aFe3PiDgCiteDkeH8KrrpZSve1pcM4SNjxwwmIKlJdrbcaJfWRsSoGFjzbFgOecISiVaJ9ZWpb89/+BeAz1Zpmu8DSyY22dG/K6ZDx5qNFg8pehdOUYY24oMamd4J2u2lUgkCKGBZMQgBZFwk+q7H86B/byGuTDEizLjGPTY/sMms1FAX55xBydxrADAer/pKrOF1v7Dq9C1Z9QVcm5D9G4DcenyWUdMyK43NXbVQLPxLOng51KO9icp2j4U7pwHP"
                ],
                "issuer": "https://login.microsoftonline.com/40a214bf-da79-471d-8daa-1a6db9ce8e22/v2.0"
            }
          ]
      }
  )))
  .expect(1)
  .mount(server)
  .await;
}

macro_rules! verify_test {
    ($fn_name:ident, $token:expr, $iat:expr, $groups_claim:expr, $client_id:expr, $expected_issuer:expr, $expect:expr) => {
        #[tokio::test]
        async fn $fn_name() {
            let client = {
                let leeway = Duration::seconds(5);
                let clock_fn = || {
                    DateTime::<Utc>::from_naive_utc_and_offset(NaiveDateTime::from_timestamp_opt($iat, 0).unwrap(), Utc)
                };
                Client {
                    time_opts: TimeOptions::new(leeway, clock_fn),
                    groups_claim: $groups_claim,
                    client_id: $client_id,
                    ..Default::default()
                }
            };
            let server = MockServer::start().await;
            let issuer_url: Url = server.uri().parse().unwrap();
            for use_oidc in [false, true] {
                let actual = if use_oidc {
                    set_up_oidc_server(&issuer_url, &server).await;
                    client
                        .verify_rs_token_using_oidc_discovery($token, &issuer_url, $expected_issuer)
                        .await
                        .unwrap()
                } else {
                    // jwks endpoint
                    let jwks_uri = issuer_url.join(JWKS_PATH).unwrap();
                    set_up_jwks_server(jwks_uri.path(), &server).await;
                    client
                        .verify_rs_token_using_jwks_endpoint($token, &jwks_uri, Some($expected_issuer))
                        .await
                        .unwrap()
                };
                assert_eq!(actual, $expect);
                server.reset().await;
            }
        }
    };
}

macro_rules! verify_fail {
    ($fn_name:ident, $token:expr, $iat:expr, $groups_claim:expr, $client_id:expr, $expected_issuer:expr, $err:literal) => {
        #[tokio::test]
        async fn $fn_name() {
            let client = {
                let leeway = Duration::seconds(5);
                let clock_fn = || {
                    DateTime::<Utc>::from_naive_utc_and_offset(NaiveDateTime::from_timestamp_opt($iat, 0).unwrap(), Utc)
                };
                Client {
                    time_opts: TimeOptions::new(leeway, clock_fn),
                    groups_claim: $groups_claim,
                    client_id: $client_id,
                    ..Default::default()
                }
            };
            let server = MockServer::start().await;
            let issuer_url: Url = server.uri().parse().unwrap();

            for use_oidc in [true] {
                let actual = if use_oidc {
                    set_up_oidc_server(&issuer_url, &server).await;
                    client
                        .verify_rs_token_using_oidc_discovery($token, &issuer_url, $expected_issuer)
                        .await
                        .unwrap_err()
                        .to_string()
                } else {
                    // jwks endpoint
                    let jwks_uri = issuer_url.join(JWKS_PATH).unwrap();
                    set_up_jwks_server(jwks_uri.path(), &server).await;
                    client
                        .verify_rs_token_using_jwks_endpoint($token, &jwks_uri, Some($expected_issuer))
                        .await
                        .unwrap_err()
                        .to_string()
                };
                assert_eq!(actual, $err);
                server.reset().await;
            }
        }
    };
}

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

verify_test!(
    basic_token,
    TOKEN,
    TOKEN_IAT,
    None,
    None,
    "https://clerk.b74v0.5y6hj.lcl.dev",
    VerifiedToken {
        identity: Some(TOKEN_SUB.to_string()),
        groups: BTreeSet::new(),
        token_claims: serde_json::from_str(
            r#"{
            "azp": "https://grafbase.dev",
            "iss": "https://clerk.b74v0.5y6hj.lcl.dev",
            "sid": "sess_2BCiGPhgXZgAV00KfPrD3KSAHCO",
            "sub": "user_25sYSVDXCrWW58OusREXyl4zp30"
        }"#
        )
        .unwrap(),
    }
);

verify_test!(
    issuer_with_additional_slash_should_verify_for_backwards_compatibility,
    TOKEN,
    TOKEN_IAT,
    None,
    None,
    "https://clerk.b74v0.5y6hj.lcl.dev/",
    VerifiedToken {
        identity: Some(TOKEN_SUB.to_string()),
        groups: BTreeSet::new(),
        token_claims: serde_json::from_str(
            r#"{
            "azp": "https://grafbase.dev",
            "iss": "https://clerk.b74v0.5y6hj.lcl.dev",
            "sid": "sess_2BCiGPhgXZgAV00KfPrD3KSAHCO",
            "sub": "user_25sYSVDXCrWW58OusREXyl4zp30"
        }"#
        )
        .unwrap(),
    }
);

verify_fail!(
    token_from_future,
    TOKEN,
    TOKEN_IAT - 10,
    None,
    None,
    "https://clerk.b74v0.5y6hj.lcl.dev",
    "invalid issue time"
);

verify_fail!(
    token_with_invalid_audience,
    TOKEN,
    TOKEN_IAT,
    None,
    Some("some-id"),
    "https://clerk.b74v0.5y6hj.lcl.dev",
    "audience does not match client ID"
);

verify_fail!(
    invalid_issuer_should_fail,
    TOKEN,
    TOKEN_IAT - 10,
    None,
    None,
    "https://example.com",
    "issuer claim mismatch"
);

/*
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
verify_test!(
        token_with_groups,
        "eyJhbGciOiJSUzI1NiIsImtpZCI6Imluc18yM2k2V0dJRFdobFBjTGVlc3hibWNVTkxaeUoiLCJ0eXAiOiJKV1QifQ.eyJleHAiOjE2NTgxNDI1MTQsImdyb3VwcyI6WyJhZG1pbiIsIm1vZGVyYXRvciJdLCJpYXQiOjE2NTgxNDE5MTQsImlzcyI6Imh0dHBzOi8vY2xlcmsuYjc0djAuNXk2aGoubGNsLmRldiIsImp0aSI6ImVjMGZmZmY3MjQzNDcyNjE3NDBiIiwibmJmIjoxNjU4MTQxOTA5LCJzdWIiOiJ1c2VyXzI1c1lTVkRYQ3JXVzU4T3VzUkVYeWw0enAzMCJ9.tnmYybDBENzLyGiSG4HFJQbTgOkx2MC4JyaywRksG-kDKLBnhfbJMwRULadzgAkQOFcmFJYsIYagK1VQ05HA4awy-Fq5WDSWyUWgde0SZTj12Fw6lKtlZp5FN8yRQI2h4l_zUMhG1Q0ZxPpzsxnAM5Y3TLVBmyxQeq5X8VdFbg24Ra5nFLXhTb3hTqCr6gmXQQ3kClseFgIWt-p57rv_7TSrnUe7dbSpNlqgcL1v3IquIlfGlIcS-G5jkkgKYwzclr3tYW3Eog0Vgm-HuCf-mvNCkZur3XA1SCaxJIoP0fNZK5DVsKfvSq574W1tzEV29DPN1i1j5CYmMU-sV-CmIA",
        1_658_141_914,
        Some("groups"),
        None,
        "https://clerk.b74v0.5y6hj.lcl.dev".parse::<url::Url>().unwrap().as_ref(),
        VerifiedToken {
            identity: Some(TOKEN_SUB.to_string()),
            groups: vec!["admin", "moderator"].into_iter().map(String::from).collect(),
            token_claims: serde_json::from_str(r#"{
                "groups": [
                    "admin",
                    "moderator"
                ],
                "iss": "https://clerk.b74v0.5y6hj.lcl.dev",
                "jti": "ec0ffff724347261740b",
                "sub": "user_25sYSVDXCrWW58OusREXyl4zp30"
            }"#).unwrap()
        }
    );

/*
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
verify_test!(
    token_with_null_groups,
    "eyJhbGciOiJSUzI1NiIsImtpZCI6Imluc18yM2k2V0dJRFdobFBjTGVlc3hibWNVTkxaeUoiLCJ0eXAiOiJKV1QifQ.eyJleHAiOjE2NjAwNDE1NzQsImdyb3VwcyI6bnVsbCwiaWF0IjoxNjYwMDQwOTc0LCJpc3MiOiJodHRwczovL2NsZXJrLmI3NHYwLjV5NmhqLmxjbC5kZXYiLCJqdGkiOiIxYzk3NmYzNTg2ZmUzNDNjMTQ2YiIsIm5iZiI6MTY2MDA0MDk2OSwic3ViIjoidXNlcl8yNXNZU1ZEWENyV1c1OE91c1JFWHlsNHpwMzAifQ.vQp09Lu_z55WnrXHxC5-sy6IXSgJfjn5RnswHC8cWWDjf6xvY8x1YsSGz0IOSBOI8-_yhSyT8YJiLsGZUblPvuiD1R91Bep3ADz107t7JV0D21FgZUSsVcp-94B4vEo84lfLWynxYGf7kJ-fFgQKH9mXvZNHpcno5-xf_Ywkdjq-IhL3LnTLdpVrVuNTyWutpPL47CMfs3W71lJJ62hmLIVV3BQIDYezb9GlPXzSI4m5Rdx72lLSVjVr41rHtqdEWXAiIQ7FiKBCrMteyUoIJ12kQowEjbCGfA58L06Jk5IHBrjXnv5-ZNNnQA7pSJ6ouOHHVeBN4zhvUdhxW1mMsg",
    1_660_040_974,
    Some("groups"),
    None,
    "https://clerk.b74v0.5y6hj.lcl.dev",
    VerifiedToken {
        identity: Some(TOKEN_SUB.to_string()),
        groups: BTreeSet::new(),
        token_claims: serde_json::from_str(r#"{
            "groups": null,
            "iss": "https://clerk.b74v0.5y6hj.lcl.dev",
            "jti": "1c976f3586fe343c146b",
            "sub": "user_25sYSVDXCrWW58OusREXyl4zp30"
        }"#).unwrap()
    }
);

/*
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
verify_test!(
        token_from_auth0,
        "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6Ii1QU3RkSUNmYXFBRmRVbkRTcTYzRSJ9.eyJodHRwczovL2dyYWZiYXNlLmNvbS9qd3QvY2xhaW1zL2dyb3VwcyI6WyJhZG1pbiJdLCJpc3MiOiJodHRwczovL2diLW9pZGMuZXUuYXV0aDAuY29tLyIsInN1YiI6IlN2WHIxeVVpdnhYMDhBampqZ3h4NDYyakpZOXdxUDFQQGNsaWVudHMiLCJhdWQiOiJodHRwczovL2dyYWZiYXNlLmNvbSIsImlhdCI6MTY2NDk2MDY3NCwiZXhwIjoxNjY1MDQ3MDc0LCJhenAiOiJTdlhyMXlVaXZ4WDA4QWpqamd4eDQ2MmpKWTl3cVAxUCIsImd0eSI6ImNsaWVudC1jcmVkZW50aWFscyJ9.HI8mxp_05-GpXHewW7_noFkUcwm0vkTf_gdmfCxh8SlNGFEZycgT_l235nfZleQ4GfsTaP0yLvpvBn5pMdHRcUnAlImvALOXAFfnYFbvwjZP0vhqfz7-vNtMdoUlOyyaxWd0idVimVPJDHmZc0lWYuUks69BdEUXyJm19XzhPodi3HtLqiF7zPOflmiOAsZjSMc5jkqVO8qv39j9WpfStr0XO97n4vGOPoA1RPenYighbethBH6tWOph2Lp7gx1HUByHQwu5GlLeDKJO-n-dAV3xAUcVKtIh_u5Yd6gofC1HTdUjWjzjrpv9SpzrqDcmzaY1WPKi-7Il17TjgXT4kA",
        1_664_960_674,
        Some("https://grafbase\\.com/jwt/claims/groups"),
        Some("https://grafbase.com"),
        "https://gb-oidc.eu.auth0.com/",
        VerifiedToken {
            identity: Some("SvXr1yUivxX08Ajjjgxx462jJY9wqP1P@clients".to_string()),
            groups: vec!["admin".to_string()].into_iter().collect(),
            token_claims: serde_json::from_str(r#"{
                "https://grafbase.com/jwt/claims/groups": [
                    "admin"
                ],
                "iss": "https://gb-oidc.eu.auth0.com/",
                "sub": "SvXr1yUivxX08Ajjjgxx462jJY9wqP1P@clients",
                "aud": "https://grafbase.com",
                "azp": "SvXr1yUivxX08Ajjjgxx462jJY9wqP1P",
                "gty": "client-credentials"
            }"#).unwrap()
        }
    );

/*
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
verify_test!(
        token_with_nested_groups,
        "eyJhbGciOiJSUzI1NiIsImtpZCI6Imluc18yRE5wbDVFQ0FwQ1NSYVNDT3V3Y1lsaXJ4QVYiLCJ0eXAiOiJKV1QifQ.eyJleHAiOjE2NjY3MTUwODMsImh0dHBzOi8vZ3JhZmJhc2UuY29tL2p3dC9jbGFpbXMiOnsieC1ncmFmYmFzZS1hbGxvd2VkLXJvbGVzIjpbImVkaXRvciIsInVzZXIiLCJtb2QiXX0sImlhdCI6MTY2NjcxNDQ4MywiaXNzIjoiaHR0cHM6Ly9jbGVyay5ncmFmYmFzZS12ZXJjZWwuZGV2IiwianRpIjoiOTE4ZjkwMzZkMWI1YWEyYTE1OWEiLCJuYmYiOjE2NjY3MTQ0NzgsInN1YiI6InVzZXJfMkU0c1Jqb2tuMnIxNFJMd2hFdmpWc0hnQ21HIn0.jA1pmbIBn_Vkos5-irFyFhwyq4OvxnkMcs8y_joWGmGnabS9I2YM5QBP-l7ZuFY9G8b5Up_Jzr0C1IsoIr0P3fM6yGdwe8MXEvZyKRXDbScq0sUvsMJTn2FJrUL0NgE-2fOVh-H0CNqDx2c584mYDgeMGXg2po_JAhszmqqLYC8KyypF2Y_j6jtyW6kiE_nbdRLINz-lEP3Wvmy60qeZHwDX4CzcME_y7avM10vTpqSoojuaoEKdCQh7tEKIpgCI0CdDx31B_bKaHPJ3nDw8fTZQ5HxK4YXkRPIdxMjG3Dby4EKuvvegZQDoASE4gUyPJ0qBgeOXUNdf5Vk6DJX9sQ",
        1_666_714_483,
        Some("https://grafbase\\.com/jwt/claims.x-grafbase-allowed-roles"),
        None,
        "https://clerk.grafbase-vercel.dev".parse::<url::Url>().unwrap().as_ref(),
        VerifiedToken {
            identity: Some("user_2E4sRjokn2r14RLwhEvjVsHgCmG".to_string()),
            groups: vec!["editor", "user", "mod"].into_iter().map(String::from).collect(),
            token_claims: serde_json::from_str(r#"{
                "https://grafbase.com/jwt/claims": {
                    "x-grafbase-allowed-roles": [
                        "editor",
                        "user",
                        "mod"
                    ]
                },
                "iss": "https://clerk.grafbase-vercel.dev",
                "jti": "918f9036d1b5aa2a159a",
                "sub": "user_2E4sRjokn2r14RLwhEvjVsHgCmG"
            }"#).unwrap()
        }
    );

/*
{
  "header": {
    "typ": "JWT",
    "alg": "HS512"
  },
  "payload": {
    "aud": [
      "app1",
      "app2"
    ],
    "exp": 1673369363,
    "groups": [
      "admin",
      "backend"
    ],
    "iat": 1673368763,
    "iss": "https://clerk.b74v0.5y6hj.lcl.dev",
    "jti": "0696be42be3fc3b2212d",
    "nbf": 1673368758,
    "sub": "user_2E7nWay3fFXh0MRgzBJZUx59UzP"
  }
}
*/
#[tokio::test]
async fn token_signed_with_secret() {
    let client = {
        let leeway = Duration::seconds(5);
        let clock_fn = || {
            DateTime::<Utc>::from_naive_utc_and_offset(
                NaiveDateTime::from_timestamp_opt(1_673_368_763, 0).unwrap(),
                Utc,
            )
        };
        Client {
            time_opts: TimeOptions::new(leeway, clock_fn),
            groups_claim: Some("groups"),
            client_id: Some("app2"),
            ..Default::default()
        }
    };

    let token = "eyJhbGciOiJIUzUxMiIsInR5cCI6IkpXVCJ9.eyJhdWQiOlsiYXBwMSIsImFwcDIiXSwiZXhwIjoxNjczMzY5MzYzLCJncm91cHMiOlsiYWRtaW4iLCJiYWNrZW5kIl0sImlhdCI6MTY3MzM2ODc2MywiaXNzIjoiaHR0cHM6Ly9jbGVyay5iNzR2MC41eTZoai5sY2wuZGV2IiwianRpIjoiMDY5NmJlNDJiZTNmYzNiMjIxMmQiLCJuYmYiOjE2NzMzNjg3NTgsInN1YiI6InVzZXJfMkU3bldheTNmRlhoME1SZ3pCSlpVeDU5VXpQIn0.x6eAgltLZqhUjT1Lr9sPLItiv0hJ4dvhuoIPMYZM4_eEB-hmmqIxxS5tdZddvDzh5jPAkwGjuynfM-WJ3Xgxcg";
    let issuer = "https://clerk.b74v0.5y6hj.lcl.dev".to_string();
    let issuer_url = issuer.parse().unwrap();
    let secret = SecretString::new("topsecret".to_string());

    assert_eq!(
        client.verify_hs_token(token, &issuer, &secret).unwrap(),
        VerifiedToken {
            identity: Some("user_2E7nWay3fFXh0MRgzBJZUx59UzP".to_string()),
            groups: vec!["admin", "backend"].into_iter().map(String::from).collect(),
            token_claims: serde_json::from_str(
                r#"{
                "aud": [
                    "app1",
                    "app2"
                ],
                "groups": [
                    "admin",
                    "backend"
                ],
                "iss": "https://clerk.b74v0.5y6hj.lcl.dev",
                "jti": "0696be42be3fc3b2212d",
                "sub": "user_2E7nWay3fFXh0MRgzBJZUx59UzP"
            }"#
            )
            .unwrap()
        }
    );

    let new_client = Client {
        client_id: Some("app3"),
        ..client
    };

    assert_eq!(
        new_client
            .verify_hs_token(token, &issuer, &secret)
            .unwrap_err()
            .to_string(),
        "audience does not match client ID"
    );

    assert_eq!(
        new_client
            .verify_rs_token_using_oidc_discovery(token, &issuer_url, &issuer)
            .await
            .unwrap_err()
            .to_string(),
        "unsupported algorithm: HS512"
    );
}

/*
{
  "header": {
    "typ": "JWT",
    "alg": "HS256",
  },
  "payload": {
    "iss": "https://idp.example.com",
    "groups": [
        null
    ],
    "exp": 1700000000,
    "sub": "user_25sYSVDXCrWW58OusREXyl4zp30",
    "iat": 1516239022
  }
}
*/
#[test]
fn token_with_groups_containing_null_should_be_interpreted_as_empty_groups() {
    let client = {
        let leeway = Duration::seconds(5);
        let clock_fn = || {
            DateTime::<Utc>::from_naive_utc_and_offset(
                NaiveDateTime::from_timestamp_opt(1_673_368_763, 0).unwrap(),
                Utc,
            )
        };
        Client {
            time_opts: TimeOptions::new(leeway, clock_fn),
            groups_claim: Some("groups"),
            client_id: None,
            ..Default::default()
        }
    };

    let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJodHRwczovL2lkcC5leGFtcGxlLmNvbSIsImdyb3VwcyI6W251bGxdLCJleHAiOjE3MDAwMDAwMDAsInN1YiI6InVzZXJfMjVzWVNWRFhDcldXNThPdXNSRVh5bDR6cDMwIiwiaWF0IjoxNTE2MjM5MDIyfQ.xG8XKXz_CmyPYfWE44m91DVsgZcNULr2GrjzkQZreac";
    let issuer = "https://idp.example.com";
    let secret = SecretString::new("abc123".to_string());

    assert_eq!(
        client.verify_hs_token(token, issuer, &secret).unwrap(),
        VerifiedToken {
            identity: Some(TOKEN_SUB.to_string()),
            groups: BTreeSet::default(),
            token_claims: serde_json::from_str(
                r#"{
                "iss": "https://idp.example.com",
                "groups": [
                    null
                ],
                "sub": "user_25sYSVDXCrWW58OusREXyl4zp30"
            }"#
            )
            .unwrap()
        }
    );
}

/*
{
  "header": {
    "typ": "JWT",
    "alg": "HS256",
  },
  "payload": {
    "iss": "https://idp.example.com",
    "groups": "wrong",
    "exp": 1700000000,
    "sub": "user_25sYSVDXCrWW58OusREXyl4zp30",
    "iat": 1516239022
  }
}
*/
#[test]
fn token_with_invalid_groups_set_to_string_should_fail() {
    let client = {
        let leeway = Duration::seconds(5);
        let clock_fn = || {
            DateTime::<Utc>::from_naive_utc_and_offset(
                NaiveDateTime::from_timestamp_opt(1_673_368_763, 0).unwrap(),
                Utc,
            )
        };
        Client {
            time_opts: TimeOptions::new(leeway, clock_fn),
            groups_claim: Some("groups"),
            client_id: None,
            ..Default::default()
        }
    };

    let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJodHRwczovL2lkcC5leGFtcGxlLmNvbSIsImdyb3VwcyI6Indyb25nIiwiZXhwIjoxNzAwMDAwMDAwLCJzdWIiOiJ1c2VyXzI1c1lTVkRYQ3JXVzU4T3VzUkVYeWw0enAzMCIsImlhdCI6MTUxNjIzOTAyMn0.0_KXw7LOmVCfQxoQeKk1tgxNb8asQWCA0VDOAENG134";
    let issuer = "https://idp.example.com";
    let secret = SecretString::new("abc123".to_string());

    assert_eq!(
        client.verify_hs_token(token, issuer, &secret).unwrap_err().to_string(),
        "invalid groups claim groups"
    );
}

/*
{
  "header": {
    "typ": "JWT",
    "alg": "HS256",
  },
  "payload": {
    "iss": "https://idp.example.com",
    "groups": ["g1", 0],
    "exp": 1700000000,
    "sub": "user_25sYSVDXCrWW58OusREXyl4zp30",
    "iat": 1516239022
  }
}
*/
#[test]
fn token_with_invalid_groups_set_to_array_of_wrong_type_should_fail() {
    let client = {
        let leeway = Duration::seconds(5);
        let clock_fn = || {
            DateTime::<Utc>::from_naive_utc_and_offset(
                NaiveDateTime::from_timestamp_opt(1_673_368_763, 0).unwrap(),
                Utc,
            )
        };
        Client {
            time_opts: TimeOptions::new(leeway, clock_fn),
            groups_claim: Some("groups"),
            client_id: None,
            ..Default::default()
        }
    };

    let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJodHRwczovL2lkcC5leGFtcGxlLmNvbSIsImdyb3VwcyI6WyJnMSIsMF0sImV4cCI6MTcwMDAwMDAwMCwic3ViIjoidXNlcl8yNXNZU1ZEWENyV1c1OE91c1JFWHlsNHpwMzAiLCJpYXQiOjE1MTYyMzkwMjJ9.JYjqAYk6OLARIU6qtoUYG4NffVgidOJCXJlOIV7yEJw";
    let issuer = "https://idp.example.com";
    let secret = SecretString::new("abc123".to_string());

    assert_eq!(
        client.verify_hs_token(token, issuer, &secret).unwrap_err().to_string(),
        "invalid groups claim groups"
    );
}

/*
{
  "header": {
    "typ": "JWT",
    "alg": "HS256",
  },
  "payload": {
    "iss": "https://idp.example.com",
    "groups": null,
    "exp": 1700000000,
    "sub": "user_25sYSVDXCrWW58OusREXyl4zp30",
    "iat": 1516239022
  }
}
*/
#[test]
fn token_with_groups_set_to_null_should_be_interpreted_as_empty_groups() {
    let client = {
        let leeway = Duration::seconds(5);
        let clock_fn = || {
            DateTime::<Utc>::from_naive_utc_and_offset(
                NaiveDateTime::from_timestamp_opt(1_673_368_763, 0).unwrap(),
                Utc,
            )
        };
        Client {
            time_opts: TimeOptions::new(leeway, clock_fn),
            groups_claim: Some("groups"),
            client_id: None,
            ..Default::default()
        }
    };

    let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJodHRwczovL2lkcC5leGFtcGxlLmNvbSIsImdyb3VwcyI6bnVsbCwiZXhwIjoxNzAwMDAwMDAwLCJzdWIiOiJ1c2VyXzI1c1lTVkRYQ3JXVzU4T3VzUkVYeWw0enAzMCIsImlhdCI6MTUxNjIzOTAyMn0.VuCT7GRY01ph_4xWlK1mqIFFx6F9Jijj4HptGajXRoU";
    let issuer = "https://idp.example.com";
    let secret = SecretString::new("abc123".to_string());

    assert_eq!(
        client.verify_hs_token(token, issuer, &secret).unwrap(),
        VerifiedToken {
            identity: Some(TOKEN_SUB.to_string()),
            groups: BTreeSet::default(),
            token_claims: serde_json::from_str(
                r#"{
                "iss": "https://idp.example.com",
                "groups": null,
                "sub": "user_25sYSVDXCrWW58OusREXyl4zp30"
            }"#
            )
            .unwrap()
        }
    );
}

/*
{
  "header": {
    "alg": "RS256",
    "kid": "71f19172-229c-4de5-9b08-c533fd2cee8a",
    "typ": "JWT"
  },
  "payload": {
    "exp": 1683725226,
    "iat": 1683721626,
    "sub": "0cbc7311-2286-4a97-a6fd-460e4ec06fa4"
  }
}
*/
const HANKO_JWT: &str = "eyJhbGciOiJSUzI1NiIsImtpZCI6IjcxZjE5MTcyLTIyOWMtNGRlNS05YjA4LWM1MzNmZDJjZWU4YSIsInR5cCI6IkpXVCJ9.eyJleHAiOjE2ODM3MjUyMjYsImlhdCI6MTY4MzcyMTYyNiwic3ViIjoiMGNiYzczMTEtMjI4Ni00YTk3LWE2ZmQtNDYwZTRlYzA2ZmE0In0.JtnAcRJBF4-Nz1O3bOsN6yyQQXJfaVdohHzptlrk0oM14IjjHIirtJBAnxoJTCY8WTqngOwliDt0YX1KiRGIXxcdJgRLXNlMVwSbLh3KRK6dax7kUZaK_MQkUuYg9j2cdWl-bl39NEvyYL8QFwRq9BBhvdHpptPtWJtzMFS7l0wjqGrItG1S91csldh0IStTZDkscTk3Kq-xjdRez7mWOFOC4v-9fTjm743Txu2MOyNUHvnPf1HjSyfk2Mxw-JulIjNIezzXG2H_Z7P9WJSZPwfaGOpfFMBr5NG6qXefrNDyWaflZoaYtnYMWh4V0HcwgmQyIG_dEWCaoRWczIruCWDidSmpKwjfl3yZaru7jV2kTdhfI2w-UKXJ-7vxYP-wmdFBUuBl18ksDYsra1qNgY4iF95M-k7wsr2thfB9JiOgmDsxVtoauO6V7T3kyuSTLf6dp5OtlsW7Ep0LssfRJ5Q86VpUG6Z54zsRTtZQCT5tjyHhM5ezLxXhTwKbuo-TSekfMd18n49k1qj0t3-_CMuDcsN_t0G0uBLoog_SJ24u9G0MZ-23INe4pqSUu5wX8iasiz4dlhZPFdLoEqi5mt2bDKny1Ys5gJDxp_AJv40DdGHqwm7QU4E1CPrp2ZwxjSnfs6Xov_HLERJOnIQaDSmQtMS-wOvoXnrNIn9G6DQ";

#[tokio::test]
async fn jwks_from_hanko_should_verify() {
    let client = {
        let leeway = Duration::seconds(5);
        let clock_fn = || {
            DateTime::<Utc>::from_naive_utc_and_offset(
                NaiveDateTime::from_timestamp_opt(1_683_721_626, 0).unwrap(),
                Utc,
            )
        };
        Client {
            time_opts: TimeOptions::new(leeway, clock_fn),
            ..Default::default()
        }
    };
    let server = MockServer::start().await;
    let issuer: Url = server.uri().parse().unwrap();

    let actual = {
        // jwks endpoint
        let jwks_uri = issuer.join(JWKS_PATH).unwrap();
        set_up_jwks_server(jwks_uri.path(), &server).await;
        client
            .verify_rs_token_using_jwks_endpoint(HANKO_JWT, &jwks_uri, None)
            .await
            .unwrap()
    };
    assert_eq!(
        actual,
        VerifiedToken {
            identity: Some("0cbc7311-2286-4a97-a6fd-460e4ec06fa4".to_string()),
            groups: BTreeSet::default(),
            token_claims: serde_json::from_str(
                r#"{
                "sub": "0cbc7311-2286-4a97-a6fd-460e4ec06fa4"
            }"#
            )
            .unwrap()
        }
    );
}

/*
{
  "header": {
    "typ": "JWT",
    "alg": "RS256",
    "kid": "-KI3Q9nNR7bRofxmeZoXqbHZGew"
  },
  "payload": {
    "aud": "61142eb9-9373-437f-a505-a983dbbffc96",
    "iss": "https://login.microsoftonline.com/40a214bf-da79-471d-8daa-1a6db9ce8e22/v2.0",
    "iat": 1683197158,
    "nbf": 1683197158,
    "exp": 1683201058,
    "aio": "AVQAq/8TAAAAVJ5ENTXzJWQ967edz4E21oESCS83p0ipuRHXdk/J3lrOXVbd5DjNd+dejephxP5uIkxyo7yRs1yg49W76ChTbH4JLbFMVVX+l6Q7WGUAgzg=",
    "name": "Nathan Lindsay",
    "oid": "525d2cc9-5105-4e2a-b697-ac95c29cefaf",
    "preferred_username": "nathan@taurean.ltd",
    "rh": "0.ATEAvxSiQHnaHUeNqhptuc6OIrkuFGFzk39DpQWpg9u__JYxANk.",
    "sub": "o80Gly744fp1k6mTjuCrqyLaZ_yoed3SSqleqBKjTR0",
    "tid": "40a214bf-da79-471d-8daa-1a6db9ce8e22",
    "uti": "Wp1L6u1_M0iaL4tTgxItAA",
    "ver": "2.0"
  }
}
*/
const AZURE_JWT: &str = "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiIsImtpZCI6Ii1LSTNROW5OUjdiUm9meG1lWm9YcWJIWkdldyJ9.eyJhdWQiOiI2MTE0MmViOS05MzczLTQzN2YtYTUwNS1hOTgzZGJiZmZjOTYiLCJpc3MiOiJodHRwczovL2xvZ2luLm1pY3Jvc29mdG9ubGluZS5jb20vNDBhMjE0YmYtZGE3OS00NzFkLThkYWEtMWE2ZGI5Y2U4ZTIyL3YyLjAiLCJpYXQiOjE2ODMxOTcxNTgsIm5iZiI6MTY4MzE5NzE1OCwiZXhwIjoxNjgzMjAxMDU4LCJhaW8iOiJBVlFBcS84VEFBQUFWSjVFTlRYekpXUTk2N2VkejRFMjFvRVNDUzgzcDBpcHVSSFhkay9KM2xyT1hWYmQ1RGpOZCtkZWplcGh4UDV1SWt4eW83eVJzMXlnNDlXNzZDaFRiSDRKTGJGTVZWWCtsNlE3V0dVQWd6Zz0iLCJuYW1lIjoiTmF0aGFuIExpbmRzYXkiLCJvaWQiOiI1MjVkMmNjOS01MTA1LTRlMmEtYjY5Ny1hYzk1YzI5Y2VmYWYiLCJwcmVmZXJyZWRfdXNlcm5hbWUiOiJuYXRoYW5AdGF1cmVhbi5sdGQiLCJyaCI6IjAuQVRFQXZ4U2lRSG5hSFVlTnFocHR1YzZPSXJrdUZHRnprMzlEcFFXcGc5dV9fSll4QU5rLiIsInN1YiI6Im84MEdseTc0NGZwMWs2bVRqdUNycXlMYVpfeW9lZDNTU3FsZXFCS2pUUjAiLCJ0aWQiOiI0MGEyMTRiZi1kYTc5LTQ3MWQtOGRhYS0xYTZkYjljZThlMjIiLCJ1dGkiOiJXcDFMNnUxX00waWFMNHRUZ3hJdEFBIiwidmVyIjoiMi4wIn0.h-YikYKJp3cafbbPRKJozMUCMkrSCeXI5GszhpX2xZ0favkCa1zl8AChEEwnC4pyQqtaUs76GFsE1r7A1aAmc7lhfPOODG7-r4__lnJNSHoveh_zFAkM6ljve3HOrCm2-HpLGF13G3N_ZsL3gniNcH2QCs3pAkvVqemh4A99MABMBWZkjHh63DasSqgFQX_ooVqKWIvfksKE6T0vDKaezg3nM4tZK1rxymmal9wz6ydxvujFeO7EWWEWFgshaShH_SWKk4zeVo_aIwiLkuggQm_Ex3M_QISSmtrW9o4fpFe3lWJvmCYkBnBmkHlpOVorYYDt99IUCkX8U-k-UhEgVg";

#[tokio::test]
async fn jwt_from_azure_ad_should_verify() {
    let client = {
        let leeway = Duration::seconds(5);
        let clock_fn = || {
            DateTime::<Utc>::from_naive_utc_and_offset(
                NaiveDateTime::from_timestamp_opt(1_683_197_158, 0).unwrap(),
                Utc,
            )
        };
        Client {
            time_opts: TimeOptions::new(leeway, clock_fn),
            ..Default::default()
        }
    };
    let server = MockServer::start().await;
    let issuer_url: Url = server.uri().parse::<url::Url>().unwrap().join("some/path/").unwrap();
    let expected_issuer = "https://login.microsoftonline.com/40a214bf-da79-471d-8daa-1a6db9ce8e22/v2.0";
    let actual = {
        set_up_oidc_server(&issuer_url, &server).await;
        client
            .verify_rs_token_using_oidc_discovery(AZURE_JWT, &issuer_url, expected_issuer)
            .await
            .unwrap()
    };
    assert_eq!(
        actual,
        VerifiedToken {
            identity: Some("o80Gly744fp1k6mTjuCrqyLaZ_yoed3SSqleqBKjTR0".to_string()),
            groups: BTreeSet::default(),
            token_claims: serde_json::from_str(r#"{
                "aud": "61142eb9-9373-437f-a505-a983dbbffc96",
                "iss": "https://login.microsoftonline.com/40a214bf-da79-471d-8daa-1a6db9ce8e22/v2.0",
                "aio": "AVQAq/8TAAAAVJ5ENTXzJWQ967edz4E21oESCS83p0ipuRHXdk/J3lrOXVbd5DjNd+dejephxP5uIkxyo7yRs1yg49W76ChTbH4JLbFMVVX+l6Q7WGUAgzg=",
                "name": "Nathan Lindsay",
                "oid": "525d2cc9-5105-4e2a-b697-ac95c29cefaf",
                "preferred_username": "nathan@taurean.ltd",
                "rh": "0.ATEAvxSiQHnaHUeNqhptuc6OIrkuFGFzk39DpQWpg9u__JYxANk.",
                "sub": "o80Gly744fp1k6mTjuCrqyLaZ_yoed3SSqleqBKjTR0",
                "tid": "40a214bf-da79-471d-8daa-1a6db9ce8e22",
                "uti": "Wp1L6u1_M0iaL4tTgxItAA",
                "ver": "2.0"
            }"#).unwrap()
        }
    );
}

#[tokio::test]
async fn oidc_discovery_should_be_available() {
    let server = MockServer::start().await;
    let server_base_url: Url = server.uri().parse().unwrap();
    assert_eq!(server_base_url.path(), "/");
    let issuer_url = server_base_url.join("some/path/").unwrap();
    let expected_discovery_url = server_base_url
        .join("some/path/.well-known/openid-configuration")
        .unwrap();

    set_up_oidc_server(&issuer_url, &server).await;
    // check that the discovery URL is at the expected path.
    let http_client = reqwest::Client::new();
    let response = http_client.get(expected_discovery_url).send().await.unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let response: serde_json::Value = response.json().await.unwrap();
    let expected = serde_json::Value::String(issuer_url.to_string());
    assert_eq!(response.get("issuer").unwrap(), &expected);
    // check that the jwks URL is accessible at the expected path.
    let expected_jwks_url = server_base_url.join("some/path/.well-known/jwks.json").unwrap();
    let response = http_client.get(expected_jwks_url).send().await.unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
}
