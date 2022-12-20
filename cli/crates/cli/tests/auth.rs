mod utils;

use std::collections::HashMap;

use serde_json::{json, Value};
use utils::consts::{JWT_PROVIDER_QUERY, JWT_PROVIDER_SCHEMA};
use utils::environment::Environment;

const ISSUER_URL: &str = "https://some.issuer.test";
const JWT_SECRET: &str = "topsecret";

#[test]
fn jwt_provider() {
    let mut env = Environment::init(4015);
    env.grafbase_init();
    env.write_schema(JWT_PROVIDER_SCHEMA);
    env.set_variables(HashMap::from([
        ("ISSUER_URL".to_string(), ISSUER_URL.to_string()),
        ("JWT_SECRET".to_string(), JWT_SECRET.to_string()),
    ]));
    env.grafbase_dev();

    let client = env.create_client();
    client.poll_endpoint(30, 300);

    // No auth header -> no authorization done in CLI
    let resp = client.gql::<Value>(json!({ "query": JWT_PROVIDER_QUERY }).to_string());
    let errors: Option<Value> = dot_get_opt!(resp, "errors");
    assert!(errors.is_none(), "errors: {errors:#?}");

    // Reject invalid token
    let client = client.with_header("Authorization", "Bearer invalid-token");
    let resp = client.gql::<Value>(json!({ "query": JWT_PROVIDER_QUERY }).to_string());
    let error: Option<String> = dot_get_opt!(resp, "errors.0.message");
    assert_eq!(error, Some("Unauthorized".to_string()), "error: {error:#?}");

    // Reject valid token with wrong group
    let token = generate_token("cli_user", &["some-group"]);
    let client = client.with_header("Authorization", &format!("Bearer {token}"));
    let resp = client.gql::<Value>(json!({ "query": JWT_PROVIDER_QUERY }).to_string());
    let error: Option<String> = dot_get_opt!(resp, "errors.0.message");
    assert_eq!(
        error,
        Some("Unauthorized to call todoCollection (missing `list` operation)".to_string()),
        "error: {error:#?}"
    );

    // Accept valid token with correct group
    let token = generate_token("cli_user", &["backend"]);
    let client = client.with_header("Authorization", &format!("Bearer {token}"));
    let resp = client.gql::<Value>(json!({ "query": JWT_PROVIDER_QUERY }).to_string());
    let errors: Option<Value> = dot_get_opt!(resp, "errors");
    assert!(errors.is_none(), "errors: {errors:#?}");
}

fn generate_token(user_id: &str, groups: &[&str]) -> String {
    use jwt_compact::{
        alg::{Hs512, Hs512Key},
        prelude::*,
    };

    #[derive(Debug, serde::Serialize)]
    struct CustomClaims<'a> {
        iss: &'a str,
        sub: &'a str,
        groups: &'a [&'a str],
    }

    let key = Hs512Key::new(JWT_SECRET.as_bytes());
    let time_opts = TimeOptions::default();
    let header = Header::default().with_token_type("JWT");
    let claims = Claims::new(CustomClaims {
        iss: ISSUER_URL,
        sub: user_id,
        groups,
    })
    .set_duration_and_issuance(&time_opts, chrono::Duration::hours(1));

    Hs512.token(header, &claims, &key).unwrap()
}
