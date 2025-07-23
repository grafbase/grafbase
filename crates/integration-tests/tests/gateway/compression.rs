use graphql_mocks::{FederatedProductsSchema, dynamic::DynamicSchema};
use integration_tests::{gateway::Gateway, runtime};
use rand::{Rng as _, distributions::Alphanumeric};

#[test]
fn supports_zstd_compression() {
    runtime().block_on(async move {
        let s: String = rand::thread_rng()
            .sample_iter(Alphanumeric)
            .take(1024)
            .map(char::from)
            .collect();

        let engine = Gateway::builder()
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

#[test]
fn does_not_compress_for_stream() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FederatedProductsSchema::default())
            .with_websocket_urls()
            .build()
            .await;

        let response = engine
            .post(
                r"
                subscription {
                    newProducts {
                        upc
                        name
                        price
                    }
                }
                ",
            )
            .header(http::header::ACCEPT_ENCODING, "zstd")
            .into_sse_stream()
            .await
            .collect()
            .await;
        insta::assert_json_snapshot!(response.messages, @r###"
        [
          {
            "data": {
              "newProducts": {
                "upc": "top-4",
                "name": "Jeans",
                "price": 44
              }
            }
          },
          {
            "data": {
              "newProducts": {
                "upc": "top-5",
                "name": "Pink Jeans",
                "price": 55
              }
            }
          }
        ]
        "###);

        let response = engine
            .post(
                r"
                subscription {
                    newProducts {
                        upc
                        name
                        price
                    }
                }
                ",
            )
            .header(http::header::ACCEPT_ENCODING, "zstd")
            .into_multipart_stream()
            .await
            .collect()
            .await;
        insta::assert_json_snapshot!(response.messages, @r###"
        [
          {
            "data": {
              "newProducts": {
                "upc": "top-4",
                "name": "Jeans",
                "price": 44
              }
            }
          },
          {
            "data": {
              "newProducts": {
                "upc": "top-5",
                "name": "Pink Jeans",
                "price": 55
              }
            }
          }
        ]
        "###);
    });
}
