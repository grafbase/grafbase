use engine::Engine;
use graphql_mocks::dynamic::DynamicSchema;
use integration_tests::{federation::EngineExt, runtime};
use rand::{Rng as _, distributions::Alphanumeric};

#[test]
fn supports_zstd_compression() {
    runtime().block_on(async move {
        let s: String = rand::thread_rng()
            .sample_iter(Alphanumeric)
            .take(1024)
            .map(char::from)
            .collect();

        let engine = Engine::builder()
            .with_subgraph(
                DynamicSchema::builder(r#"type Query { str: String! }"#)
                    .with_resolver("Query", "str", serde_json::Value::String(s.clone()))
                    .into_subgraph("x"),
            )
            .build()
            .await;

        let response = engine
            .raw_execute(
                http::Request::builder()
                    .uri("http://localhost/graphql")
                    .method(http::Method::POST)
                    .header(http::header::CONTENT_TYPE, "application/json")
                    .header(http::header::ACCEPT_ENCODING, "zstd")
                    .body(Vec::from(br###"{"query":"{ str }"}"###))
                    .unwrap(),
            )
            .await;

        let body = zstd::decode_all(response.body().as_ref()).unwrap();
        let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, serde_json::json!({"data": {"str": s}}));
    })
}
