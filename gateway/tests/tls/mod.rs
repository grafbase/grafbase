use std::net::SocketAddr;

use crate::{GatewayRunner, load_schema};
use indoc::formatdoc;
use reqwest::Client;

#[test]
fn server_ceritifcates() {
    let local_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    let cert = reqwest::Certificate::from_pem(
        &std::fs::read(format!("{local_dir}/tests/tls/certificates/root-CA-crt.pem")).unwrap(),
    )
    .unwrap();

    let config = formatdoc! {r#"
        [tls]
        certificate = "{local_dir}/tests/tls/certificates/server-crt.pem"
        key = "{local_dir}/tests/tls/certificates/server-key.pem"
    "#};

    GatewayRunner::with_schema(load_schema("big"))
        .with_config(config)
        .run(async |addr: SocketAddr| {
            let client = Client::builder().add_root_certificate(cert).build().unwrap();

            let resp = client
                .post(format!("https://localhost:{}/graphql", addr.port()))
                .header("Content-Type", "application/json")
                .body(r#"{"query": "{ __typename }"}"#)
                .send()
                .await
                .unwrap();

            let status = resp.status();
            let body: serde_json::Value = resp.json().await.unwrap();

            insta::assert_json_snapshot!(body, @r#"
            {
              "data": {
                "__typename": "Query"
              }
            }
            "#);

            assert_eq!(status, http::StatusCode::OK);
        });
}
