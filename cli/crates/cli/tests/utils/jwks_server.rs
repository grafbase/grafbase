use axum::{routing::get, Json, Router};
use chrono::{Duration, Utc};
use jwt_compact::{alg::SecretBytes, jwk::JsonWebKey};
use serde_json::json;
use tokio::task::JoinSet;

/// Really basic implementation of an identity provider
pub struct IdentityServer {
    jwk: JsonWebKey<'static>,
    port: u16,
    join_set: JoinSet<()>,
}

const ISSUER: &str = "grafbase-cli-tests";
const AUDIENCE: &str = "grafbase-cli-test-server";

impl IdentityServer {
    pub async fn new() -> Self {
        let mut join_set = JoinSet::new();
        let jwk = JsonWebKey::Symmetric {
            secret: SecretBytes::borrowed(b"notreallyverysecret"),
        };

        let jwks_json = json!({
            "keys": [
                jwk
            ]
        });

        let app = Router::new().route(
            "/.well-known/jwks.json",
            get(move || std::future::ready(Json(jwks_json.clone()))),
        );

        let tcp_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = tcp_listener.local_addr().unwrap().port();

        join_set.spawn(async move {
            axum::serve(tcp_listener, app).await.unwrap();
        });

        // Give the server time to start
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;

        IdentityServer { jwk, port, join_set }
    }

    pub fn jwks_url(&self) -> String {
        format!("http://localhost:{}/.well-known/jwks.json", self.port)
    }

    pub fn ts_auth_provider(&self) -> String {
        format!(
            r#"auth.JWT({{jwks: {{url: "{}", issuer: "{ISSUER}", audience: "{AUDIENCE}"}}}}),"#,
            self.jwks_url()
        )
    }

    pub fn token(&self) -> String {
        self.token_with_claims(json!({}))
    }

    pub fn token_with_claims(&self, mut claims: serde_json::Value) -> String {
        use jwt_compact::alg::Hs256;
        use jwt_compact::prelude::*;

        let serde_json::Value::Object(claim_entries) = &mut claims else {
            panic!("Claims must be an object");
        };
        claim_entries.entry("iss").or_insert(json!(ISSUER));
        claim_entries.entry("aud").or_insert(json!(AUDIENCE));

        let header = Header::empty();
        let claims = Claims::new(claims)
            .set_duration_and_issuance(&Default::default(), Duration::minutes(5))
            .set_not_before(Utc::now() - Duration::hours(1));

        Hs256.token(&header, &claims, &(&self.jwk).try_into().unwrap()).unwrap()
    }

    pub fn auth_header(&self) -> String {
        format!("Bearer {}", self.token())
    }

    pub fn auth_header_with_claims(&self, claims: serde_json::Value) -> String {
        format!("Bearer {}", self.token_with_claims(claims))
    }
}
