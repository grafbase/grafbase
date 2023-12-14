#![allow(unused_crate_dependencies)]
mod utils;

use backend::project::GraphType;
use jwt_compact::alg::{Hs512, Hs512Key};
use jwt_compact::alg::{Rsa, RsaPrivateKey, RsaPublicKey, StrongAlg, StrongKey};
use jwt_compact::jwk::JsonWebKey;
use jwt_compact::prelude::*;
use jwt_compact::Algorithm;
use serde_derive::Serialize;
use serde_json::{json, Value};
use std::borrow::Cow;
use std::collections::HashMap;
use url::Url;
use utils::async_client::AsyncClient;
use utils::consts::{
    AUTH_JWKS_PROVIDER_WITH_ENDPOINT_SCHEMA, AUTH_JWKS_PROVIDER_WITH_ISSUER_ENDPOINT_SCHEMA,
    AUTH_JWKS_PROVIDER_WITH_ISSUER_SCHEMA, AUTH_JWT_PROVIDER_SCHEMA, AUTH_OIDC_PROVIDER_SCHEMA,
    AUTH_PUBLIC_GLOBAL_SCHEMA, AUTH_PUBLIC_TYPE_SCHEMA, AUTH_QUERY_TODOS, AUTH_TYPE_FIELD_RESOLVER_SCHEMA,
    INTROSPECTION_QUERY,
};
use utils::environment::Environment;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::utils::consts::{
    AUTHORIZER_SCHEMA, AUTH_CREATE_MUTATION, AUTH_ENTRYPOINT_FIELD_RESOLVER_SCHEMA, AUTH_ENTRYPOINT_MUTATION_TEXT,
    AUTH_ENTRYPOINT_QUERY_TEXT, AUTH_QUERY_WITH_TEXT,
};

const JWT_ISSUER_URL: &str = "https://some.issuer.test";
const JWT_SECRET: &str = "topsecret";

#[ignore]
#[test]
fn jwt_provider() {
    let mut env = Environment::init();
    env.grafbase_init(GraphType::Single);
    env.write_schema(AUTH_JWT_PROVIDER_SCHEMA);
    env.set_variables(HashMap::from([
        ("ISSUER_URL".to_string(), JWT_ISSUER_URL.to_string()),
        ("JWT_SECRET".to_string(), JWT_SECRET.to_string()),
    ]));
    env.grafbase_dev();

    let client = env.create_client();
    client.poll_endpoint(30, 300);

    // No auth header -> fail
    let resp = client.gql::<Value>(AUTH_QUERY_TODOS).send();
    let error: String = dot_get_opt!(resp, "errors.0.message").expect("should end with an auth failure");
    assert!(error.contains("Unauthorized"), "error: {error:#?}");

    // introspection does not need an auth header locally.
    insta::assert_json_snapshot!("introspection", client.gql::<Value>(INTROSPECTION_QUERY).send());

    // Reject invalid token
    let client = client.with_header("Authorization", "Bearer invalid-token");
    let resp = client.gql::<Value>(AUTH_QUERY_TODOS).send();
    let error: Option<String> = dot_get_opt!(resp, "errors.0.message");
    assert_eq!(error, Some("Unauthorized".to_string()), "error: {error:#?}");

    // Reject valid token with wrong group
    let token = generate_hs512_token("cli_user", &["some-group"]);
    let client = client.with_header("Authorization", &format!("Bearer {token}"));
    let resp = client.gql::<Value>(AUTH_QUERY_TODOS).send();
    let error: String = dot_get_opt!(resp, "errors.0.message").expect("should end with an auth failure");
    assert!(error.contains("Unauthorized"), "error: {error:#?}");

    // Accept valid token with correct group
    let token = generate_hs512_token("cli_user", &["backend"]);
    let client = client.with_header("Authorization", &format!("Bearer {token}"));
    let resp = client.gql::<Value>(AUTH_QUERY_TODOS).send();
    let errors: Option<Value> = dot_get_opt!(resp, "errors");
    assert!(errors.is_none(), "errors: {errors:#?}");

    // accept authorization via an API key
    let client = client.with_cleared_headers().with_api_key();
    let resp = client.gql::<Value>(AUTH_QUERY_TODOS).send();
    let errors: Option<Value> = dot_get_opt!(resp, "errors");
    assert!(errors.is_none(), "errors: {errors:#?}");
}

fn generate_hs512_token(sub: &str, groups: &[&str]) -> String {
    #[derive(Debug, Serialize)]
    struct CustomClaims<'a> {
        iss: &'a str,
        sub: &'a str,
        groups: &'a [&'a str],
    }

    let key = Hs512Key::new(JWT_SECRET.as_bytes());
    let time_opts = TimeOptions::default();
    let header: jwt_compact::Header<jwt_compact::Empty> = Header::default().with_token_type("JWT");
    let claims = Claims::new(CustomClaims {
        iss: JWT_ISSUER_URL,
        sub,
        groups,
    })
    .set_duration_and_issuance(&time_opts, chrono::Duration::hours(1));

    Hs512.token(&header, &claims, &key).unwrap()
}

const OIDC_DISCOVERY_PATH: &str = ".well-known/openid-configuration";
const JWKS_PATH: &str = ".well-known/jwks.json";

async fn set_up_oidc_server(issuer: &Url, server: &MockServer, key_set: JsonWebKeySet<'_>) {
    let discovery_url = issuer.join(OIDC_DISCOVERY_PATH).unwrap();
    let jwks_uri = issuer.join(JWKS_PATH).unwrap();
    Mock::given(method("GET"))
        .and(path(discovery_url.path()))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!(
            { "issuer": issuer, "jwks_uri": jwks_uri }
        )))
        .expect(0..)
        .mount(server)
        .await;

    set_up_jwks_server(jwks_uri.path(), server, key_set).await;
}

async fn set_up_jwks_server(jwks_path: &str, server: &MockServer, key_set: JsonWebKeySet<'_>) {
    Mock::given(method("GET"))
        .and(path(jwks_path))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::to_value(key_set).unwrap()))
        .expect(0..)
        .mount(server)
        .await;
}

fn to_verifying_key_set<'a>(pub_key: &'a StrongKey<RsaPublicKey>, kid: &'a str) -> JsonWebKeySet<'a> {
    use rsa::traits::PublicKeyParts;
    let modulus = Cow::Owned(pub_key.as_ref().n().to_bytes_be());
    let public_exponent = Cow::Owned(pub_key.as_ref().e().to_bytes_be());
    JsonWebKeySet {
        keys: vec![ExtendedJsonWebKey {
            base: JsonWebKey::Rsa {
                modulus,
                public_exponent,
                private_parts: None,
            },
            kid,
            alg: RS_256.name().to_string(),
            r#use: "sig",
        }],
    }
}

fn generate_rs256_token(
    key: &StrongKey<RsaPrivateKey>,
    sub: &str,
    groups: &[&str],
    issuer: Option<&str>,
    key_id: &str,
) -> String {
    #[derive(Debug, Serialize)]
    struct CustomClaims<'a> {
        #[serde(skip_serializing_if = "Option::is_none")]
        iss: Option<&'a str>,
        sub: &'a str,
        groups: &'a [&'a str],
    }

    let time_opts = TimeOptions::default();
    let header: jwt_compact::Header<jwt_compact::Empty> = Header::default().with_token_type("JWT").with_key_id(key_id);
    let claims = Claims::new(CustomClaims {
        iss: issuer,
        sub,
        groups,
    })
    .set_duration_and_issuance(&time_opts, chrono::Duration::hours(1));

    RS_256.token(&header, &claims, key).unwrap()
}

// A wrapper around JsonWebKey that makes the kid accessible
#[derive(Serialize, Debug)]
struct ExtendedJsonWebKey<'a> {
    #[serde(flatten)]
    base: JsonWebKey<'a>,
    kid: &'a str,
    alg: String,
    r#use: &'a str,
}

#[derive(Serialize, Debug)]
struct JsonWebKeySet<'a> {
    keys: Vec<ExtendedJsonWebKey<'a>>,
}

const RS_256: StrongAlg<Rsa> = StrongAlg(Rsa::rs256());
const KEY_ID: &str = "some-key-id";

struct SetUpOidc {
    client: AsyncClient,
    priv_key: StrongKey<RsaPrivateKey>,
    issuer_url: Url,
    #[allow(unused)] // guards the lifecycle of a test
    env: Environment,
}

async fn set_up_oidc() -> SetUpOidc {
    set_up_oidc_with_path(None).await
}

async fn set_up_oidc_with_path(path: Option<&str>) -> SetUpOidc {
    let mut rng = rand::thread_rng();
    let (priv_key, pub_key) =
        Rsa::generate(&mut rng, jwt_compact::alg::ModulusBits::TwoKibibytes).expect("key must be generated");
    let key_set = to_verifying_key_set(&pub_key, KEY_ID);
    let server = MockServer::start().await;
    let issuer_url: Url = {
        let issuer_url = server.uri().parse().unwrap();
        match path {
            None => issuer_url,
            Some(path) => {
                let path = if path.ends_with('/') {
                    path.to_string()
                } else {
                    format!("{path}/")
                };
                issuer_url.join(&path).unwrap()
            }
        }
    };
    set_up_oidc_server(&issuer_url, &server, key_set).await;
    let mut env = Environment::init_async().await;
    env.grafbase_init(GraphType::Single);
    env.write_schema(AUTH_OIDC_PROVIDER_SCHEMA);
    env.set_variables(HashMap::from([("ISSUER_URL".to_string(), issuer_url.to_string())]));
    env.grafbase_dev();
    let client = env.create_async_client();
    client.poll_endpoint(30, 300).await;
    SetUpOidc {
        client,
        priv_key,
        issuer_url,
        env,
    }
}

async fn set_up_jwks<F: Fn(&Url) -> HashMap<String, String>>(
    schema: &str,
    jwks_path: &str,
    variables_fn: F,
) -> SetUpOidc {
    let mut rng = rand::thread_rng();
    let (priv_key, pub_key) =
        Rsa::generate(&mut rng, jwt_compact::alg::ModulusBits::TwoKibibytes).expect("key must be generated");
    let key_set = to_verifying_key_set(&pub_key, KEY_ID);
    let server = MockServer::start().await;
    let issuer_url: Url = server.uri().parse().unwrap();
    let jwks_uri = issuer_url.join(jwks_path).unwrap();
    set_up_jwks_server(jwks_uri.path(), &server, key_set).await;
    let mut env = Environment::init_async().await;
    env.grafbase_init(GraphType::Single);
    env.write_schema(schema);
    env.set_variables(variables_fn(&issuer_url));
    env.grafbase_dev();
    let client = env.create_async_client();
    client.poll_endpoint(30, 300).await;
    SetUpOidc {
        client,
        priv_key,
        issuer_url,
        env,
    }
}

#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn oidc_without_token_should_only_allow_introspection() {
    let set_up = set_up_oidc().await;

    // No auth header -> fail
    let resp = set_up.client.gql::<Value>(AUTH_QUERY_TODOS).await;
    let error: String = dot_get_opt!(resp, "errors.0.message").expect("response should contain an error");
    assert!(error.contains("Unauthorized"), "error: {error:#?}");

    // introspection does not need an auth header locally.
    insta::assert_json_snapshot!("introspection", set_up.client.gql::<Value>(INTROSPECTION_QUERY).await);
}

#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn oidc_token_with_valid_group_should_work() {
    let set_up = set_up_oidc().await;

    let token = generate_rs256_token(
        &set_up.priv_key,
        "cli_user",
        &["backend"],
        Some(set_up.issuer_url.as_str()),
        KEY_ID,
    );
    let client = set_up.client.with_header("Authorization", &format!("Bearer {token}"));
    let resp = client.gql::<Value>(AUTH_QUERY_TODOS).await;
    let errors: Option<Value> = dot_get_opt!(resp, "errors");
    assert!(errors.is_none(), "errors: {errors:#?}");
}

#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn oidc_with_path_with_trailing_slash_should_work() {
    let set_up = set_up_oidc_with_path(Some("some/path/")).await;
    let token = generate_rs256_token(
        &set_up.priv_key,
        "cli_user",
        &["backend"],
        Some(set_up.issuer_url.as_str()),
        KEY_ID,
    );
    let client = set_up.client.with_header("Authorization", &format!("Bearer {token}"));
    let resp = client.gql::<Value>(AUTH_QUERY_TODOS).await;
    let errors: Option<Value> = dot_get_opt!(resp, "errors");
    assert!(errors.is_none(), "errors: {errors:#?}");
}

#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn oidc_with_path_without_trailing_slash_should_work() {
    let set_up = set_up_oidc_with_path(Some("some/path")).await;
    let token = generate_rs256_token(
        &set_up.priv_key,
        "cli_user",
        &["backend"],
        Some(set_up.issuer_url.as_str()),
        KEY_ID,
    );
    let client = set_up.client.with_header("Authorization", &format!("Bearer {token}"));
    let resp = client.gql::<Value>(AUTH_QUERY_TODOS).await;
    let errors: Option<Value> = dot_get_opt!(resp, "errors");
    assert!(errors.is_none(), "errors: {errors:#?}");
}

#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn oidc_token_with_wrong_group_should_fail() {
    let set_up = set_up_oidc().await;

    let token = generate_rs256_token(
        &set_up.priv_key,
        "cli_user",
        &["some-group"],
        Some(set_up.issuer_url.as_str()),
        KEY_ID,
    );
    let client = set_up.client.with_header("Authorization", &format!("Bearer {token}"));
    let resp = client.gql::<Value>(AUTH_QUERY_TODOS).await;
    let error: Option<String> = dot_get_opt!(resp, "errors.0.message");
    assert_eq!(
        error,
        Some("Unauthorized to access Query.todoCollection (missing list operation)".to_string()),
        "error: {error:#?}"
    );
}

#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn oidc_token_with_wrong_kid_should_fail() {
    let set_up = set_up_oidc().await;

    let token = generate_rs256_token(
        &set_up.priv_key,
        "cli_user",
        &["some-group"],
        Some(set_up.issuer_url.as_str()),
        "other-id",
    );
    let client = set_up.client.with_header("Authorization", &format!("Bearer {token}"));
    let resp = client.gql::<Value>(AUTH_QUERY_TODOS).await;
    let error: Option<String> = dot_get_opt!(resp, "errors.0.message");
    assert_eq!(error, Some("Unauthorized".to_string()));
}

#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn jwks_issuer_token_with_valid_group_should_work() {
    let set_up = set_up_jwks(AUTH_JWKS_PROVIDER_WITH_ISSUER_SCHEMA, JWKS_PATH, |issuer_url: &Url| {
        HashMap::from([("ISSUER_URL".to_string(), issuer_url.to_string())])
    })
    .await;

    let token = generate_rs256_token(
        &set_up.priv_key,
        "cli_user",
        &["backend"],
        Some(set_up.issuer_url.as_str()),
        KEY_ID,
    );
    let client = set_up.client.with_header("Authorization", &format!("Bearer {token}"));
    let resp = client.gql::<Value>(AUTH_QUERY_TODOS).await;
    let errors: Option<Value> = dot_get_opt!(resp, "errors");
    assert!(errors.is_none(), "errors: {errors:#?}");
}

#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn jwks_endoint_token_with_valid_group_should_work() {
    const ENDPOINT_PATH: &str = "custom/jwks.json";
    let set_up = set_up_jwks(
        AUTH_JWKS_PROVIDER_WITH_ENDPOINT_SCHEMA,
        ENDPOINT_PATH,
        |issuer_url: &Url| {
            HashMap::from([(
                "JWKS_ENDPOINT_URL".to_string(),
                issuer_url.join(ENDPOINT_PATH).unwrap().to_string(),
            )])
        },
    )
    .await;

    let token = generate_rs256_token(
        &set_up.priv_key,
        "cli_user",
        &["backend"],
        Some(set_up.issuer_url.as_str()),
        KEY_ID,
    );
    let client = set_up.client.with_header("Authorization", &format!("Bearer {token}"));
    let resp = client.gql::<Value>(AUTH_QUERY_TODOS).await;
    let errors: Option<Value> = dot_get_opt!(resp, "errors");
    assert!(errors.is_none(), "errors: {errors:#?}");
}

#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn jwks_endoint_and_issuer_token_with_valid_group_should_work() {
    const ENDPOINT_PATH: &str = "custom/jwks.json";
    let set_up = set_up_jwks(
        AUTH_JWKS_PROVIDER_WITH_ISSUER_ENDPOINT_SCHEMA,
        ENDPOINT_PATH,
        |issuer_url: &Url| {
            HashMap::from([
                ("ISSUER_URL".to_string(), issuer_url.to_string()),
                (
                    "JWKS_ENDPOINT_URL".to_string(),
                    issuer_url.join(ENDPOINT_PATH).unwrap().to_string(),
                ),
            ])
        },
    )
    .await;

    let token = generate_rs256_token(
        &set_up.priv_key,
        "cli_user",
        &["backend"],
        Some(set_up.issuer_url.as_str()),
        KEY_ID,
    );
    let client = set_up.client.with_header("Authorization", &format!("Bearer {token}"));
    let resp = client.gql::<Value>(AUTH_QUERY_TODOS).await;
    let errors: Option<Value> = dot_get_opt!(resp, "errors");
    assert!(errors.is_none(), "errors: {errors:#?}");
}

#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn public_global() {
    let mut env = Environment::init_async().await;
    env.grafbase_init(GraphType::Single);
    env.write_schema(AUTH_PUBLIC_GLOBAL_SCHEMA);
    env.set_variables(HashMap::from([
        ("ISSUER_URL".to_string(), JWT_ISSUER_URL.to_string()),
        ("JWT_SECRET".to_string(), JWT_SECRET.to_string()),
    ]));
    env.grafbase_dev();
    let client = env.create_async_client();
    client.poll_endpoint(30, 300).await;
    let resp = client.gql::<Value>(AUTH_QUERY_TODOS).await;
    insta::assert_json_snapshot!("public_global", resp);
}

#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn public_type() {
    let mut env = Environment::init_async().await;
    env.grafbase_init(GraphType::Single);
    env.write_schema(AUTH_PUBLIC_TYPE_SCHEMA);
    env.set_variables(HashMap::from([
        ("ISSUER_URL".to_string(), JWT_ISSUER_URL.to_string()),
        ("JWT_SECRET".to_string(), JWT_SECRET.to_string()),
    ]));
    env.grafbase_dev();
    let client = env.create_async_client();
    client.poll_endpoint(30, 300).await;
    let resp = client.gql::<Value>(AUTH_QUERY_TODOS).await;
    insta::assert_json_snapshot!("public_type", resp);
}

const RESOLVER_FILE_NAME: &str = "return-text.js";
const RESOLVER_CONTENT: &str = r#"export default function Resolver(parent, args, context, info) {
    return "Lorem ipsum dolor sit amet";
}"#;

#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn type_field_resolver_mixed() {
    let mut env = Environment::init_async().await;
    env.grafbase_init(GraphType::Single);
    env.write_schema(AUTH_TYPE_FIELD_RESOLVER_SCHEMA);
    env.write_resolver(RESOLVER_FILE_NAME, RESOLVER_CONTENT);
    env.set_variables(HashMap::from([
        ("ISSUER_URL".to_string(), JWT_ISSUER_URL.to_string()),
        ("JWT_SECRET".to_string(), JWT_SECRET.to_string()),
    ]));
    env.grafbase_dev();
    let token = generate_hs512_token("cli_user", &[]);
    let private_client = env
        .create_async_client()
        .with_header("Authorization", &format!("Bearer {token}"));
    private_client.poll_endpoint(30, 300).await;
    let public_client = env.create_async_client();
    insta::assert_json_snapshot!(
        "type_field_resolver_mixed__public_mutation_should_fail",
        public_client.gql::<Value>(AUTH_CREATE_MUTATION).await
    );

    insta::assert_json_snapshot!(
        "type_field_resolver_mixed__private_mutation_should_succeed",
        private_client.gql::<Value>(AUTH_CREATE_MUTATION).await
    );
    insta::assert_json_snapshot!(
        "type_field_resolver_mixed__private_query_should_succeed",
        private_client.gql::<Value>(AUTH_QUERY_WITH_TEXT).await
    );
    insta::assert_json_snapshot!(
        "type_field_resolver_mixed__public_query_should_fail",
        public_client.gql::<Value>(AUTH_QUERY_WITH_TEXT).await
    );
    insta::assert_json_snapshot!(
        "type_field_resolver_mixed__public_partial_query_should_succeed",
        public_client.gql::<Value>(AUTH_QUERY_TODOS).await
    );
}

#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn entrypoint_query_field_resolver() {
    let mut env = Environment::init_async().await;
    env.grafbase_init(GraphType::Single);
    env.write_schema(AUTH_ENTRYPOINT_FIELD_RESOLVER_SCHEMA);
    env.write_resolver(RESOLVER_FILE_NAME, RESOLVER_CONTENT);
    env.set_variables(HashMap::from([
        ("ISSUER_URL".to_string(), JWT_ISSUER_URL.to_string()),
        ("JWT_SECRET".to_string(), JWT_SECRET.to_string()),
    ]));
    env.grafbase_dev();
    let public_client = env.create_async_client();
    public_client.poll_endpoint(30, 300).await;
    insta::assert_json_snapshot!(
        "entrypoint_query_field_resolver__public_entrypoint_field_query_should_fail",
        public_client.gql::<Value>(AUTH_ENTRYPOINT_QUERY_TEXT).await
    );
    {
        let token = generate_hs512_token("cli_user", &["reader"]);
        let reader_client = env
            .create_async_client()
            .with_header("Authorization", &format!("Bearer {token}"));
        insta::assert_json_snapshot!(
            "entrypoint_query_field_resolver__reader_with_entrypoint_field_query_should_succeed",
            reader_client.gql::<Value>(AUTH_ENTRYPOINT_QUERY_TEXT).await
        );
    }
    {
        let token = generate_hs512_token("cli_user", &["writer"]);
        let writer_client = env
            .create_async_client()
            .with_header("Authorization", &format!("Bearer {token}"));
        insta::assert_json_snapshot!(
            "entrypoint_query_field_resolver__writer_with_entrypoint_field_query_should_fail",
            writer_client.gql::<Value>(AUTH_ENTRYPOINT_QUERY_TEXT).await
        );
    }
}

#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn entrypoint_mutation_field_resolver_mixed() {
    let mut env = Environment::init_async().await;
    env.grafbase_init(GraphType::Single);
    env.write_schema(AUTH_ENTRYPOINT_FIELD_RESOLVER_SCHEMA);
    env.write_resolver(RESOLVER_FILE_NAME, RESOLVER_CONTENT);
    env.set_variables(HashMap::from([
        ("ISSUER_URL".to_string(), JWT_ISSUER_URL.to_string()),
        ("JWT_SECRET".to_string(), JWT_SECRET.to_string()),
    ]));
    env.grafbase_dev();
    let public_client = env.create_async_client();
    public_client.poll_endpoint(30, 300).await;
    insta::assert_json_snapshot!(
        "entrypoint_mutation_field_resolver_mixed__public_entrypoint_field_mutation_should_fail",
        public_client.gql::<Value>(AUTH_ENTRYPOINT_MUTATION_TEXT).await
    );

    {
        let token = generate_hs512_token("cli_user", &["writer"]);
        let writer_client = env
            .create_async_client()
            .with_header("Authorization", &format!("Bearer {token}"));
        insta::assert_json_snapshot!(
            "entrypoint_mutation_field_resolver_mixed___writer_with_entrypoint_field_mutation_should_succeed",
            writer_client.gql::<Value>(AUTH_ENTRYPOINT_MUTATION_TEXT).await
        );
    }
    {
        let token = generate_hs512_token("cli_user", &["reader"]);
        let reader_client = env
            .create_async_client()
            .with_header("Authorization", &format!("Bearer {token}"));
        insta::assert_json_snapshot!(
            "entrypoint_mutation_field_resolver_mixed__reader_entrypoint_field_mutation_should_fail",
            reader_client.gql::<Value>(AUTH_ENTRYPOINT_MUTATION_TEXT).await
        );
    }
}

#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn authorizer_with_no_headers_should_work() {
    let mut env = Environment::init_async().await;
    env.grafbase_init(GraphType::Single);
    env.write_schema(AUTHORIZER_SCHEMA);
    let authorizer_name = "a1";
    let authorizer_content = r"export default function(context) {
        return { identity: { sub:'user1', groups: ['backend'] } };
    }";

    env.write_authorizer(format!("{authorizer_name}.js"), authorizer_content);
    env.set_variables(HashMap::from([(
        "AUTHORIZER_NAME".to_string(),
        authorizer_name.to_string(),
    )]));
    env.grafbase_dev();
    let client = env.create_async_client();
    client.poll_endpoint(30, 300).await;
    insta::assert_json_snapshot!(
        "authorizer_with_no_headers_should_work__todo_creation_should_succeed",
        client.gql::<Value>(AUTH_CREATE_MUTATION).await
    );
}

#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn authorizer_with_headers_should_work() {
    let mut env = Environment::init_async().await;
    env.grafbase_init(GraphType::Single);
    env.write_schema(AUTHORIZER_SCHEMA);
    let authorizer_name = "a1";
    let authorizer_content = r"export default function(context) {
        return { identity: { groups: [context.request.headers['h1']] } };
    }";

    env.write_authorizer(format!("{authorizer_name}.js"), authorizer_content);
    env.set_variables(HashMap::from([(
        "AUTHORIZER_NAME".to_string(),
        authorizer_name.to_string(),
    )]));
    env.grafbase_dev();
    let client = env.create_async_client().with_header("h1", "backend");
    client.poll_endpoint(30, 300).await;
    insta::assert_json_snapshot!(
        "authorizer_with_headers_should_work__todo_creation_should_succeed",
        client.gql::<Value>(AUTH_CREATE_MUTATION).await
    );
}

#[ignore]
#[tokio::test(flavor = "multi_thread")]
async fn authorizer_with_public_access_should_work() {
    let mut env = Environment::init_async().await;
    env.grafbase_init(GraphType::Single);
    env.write_schema(AUTHORIZER_SCHEMA);
    let authorizer_name = "a1";
    let authorizer_content = r"export default function(context) {
        const grp = context.request.headers['h1']
        if (grp) {
            return { identity: { groups: [grp] } };
        } else {
            return {}; // missing identity = public access
        }
    }";

    env.write_authorizer(format!("{authorizer_name}.js"), authorizer_content);
    env.set_variables(HashMap::from([(
        "AUTHORIZER_NAME".to_string(),
        authorizer_name.to_string(),
    )]));
    env.grafbase_dev();
    let private_client = env.create_async_client().with_header("h1", "backend");
    private_client.poll_endpoint(30, 300).await;
    insta::assert_json_snapshot!(
        "authorizer_with_public_access_should_work__todo_creation_should_succeed",
        private_client.gql::<Value>(AUTH_CREATE_MUTATION).await
    );
    insta::assert_json_snapshot!(
        "authorizer_with_public_access_should_work__todo_list_should_succeed",
        private_client.gql::<Value>(AUTH_QUERY_TODOS).await
    );
    let public_client = env.create_async_client();
    insta::assert_json_snapshot!(
        "authorizer_with_public_access_should_work__todo_creation_with_public_access_should_fail",
        public_client.gql::<Value>(AUTH_CREATE_MUTATION).await
    );
    insta::assert_json_snapshot!(
        "authorizer_with_public_access_should_work__todo_list_should_succeed",
        private_client.gql::<Value>(AUTH_QUERY_TODOS).await
    );
}
