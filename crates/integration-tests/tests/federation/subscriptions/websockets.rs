use engine::Engine;
use futures::StreamExt as _;
use graphql_mocks::FederatedProductsSchema;
use integration_tests::{federation::EngineExt as _, runtime};

#[test]
fn custom_websocket_path() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FederatedProductsSchema)
            .with_websocket_urls()
            .with_toml_config(
                r#"
               [graph]
               websocket_path = "/web3socket"
            "#,
            )
            .build()
            .await;

        // Default path.
        let Err(err) = engine.ws("subscription { newProducts { upc } }").await else {
            panic!("Expected a 404 response, got a stream");
        };

        assert_eq!(err.to_string(), "HTTP error: 404 Not Found");

        // Custom path.
        let mut stream = engine
            .ws("subscription { newProducts { upc } }")
            .with_path("/web3socket")
            .await
            .unwrap();

        let first = stream.next().await.unwrap();
        let second = stream.next().await.unwrap();
        assert!(stream.next().await.is_none());

        insta::assert_json_snapshot!([first, second], @r#"
        [
          {
            "data": {
              "newProducts": {
                "upc": "top-4"
              }
            }
          },
          {
            "data": {
              "newProducts": {
                "upc": "top-5"
              }
            }
          }
        ]
        "#);
    });
}

#[test]
fn websockets_basic_no_init_payload() {
    let (first, second) = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FederatedProductsSchema)
            .with_websocket_urls()
            .build()
            .await;

        let mut stream = engine.ws("subscription { newProducts { upc } }").await.unwrap();

        let first = stream.next().await.unwrap();
        let second = stream.next().await.unwrap();
        assert!(stream.next().await.is_none());

        (first, second)
    });

    insta::assert_json_snapshot!([first, second], @r#"
    [
      {
        "data": {
          "newProducts": {
            "upc": "top-4"
          }
        }
      },
      {
        "data": {
          "newProducts": {
            "upc": "top-5"
          }
        }
      }
    ]
    "#);
}

#[test]
fn websockets_forward_subgraph_headers() {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FederatedProductsSchema)
            .with_websocket_urls()
            .with_toml_config(
                r#"
            [[headers]]
            rule = "forward"
            name = "authorization"
            "#,
            )
            .build()
            .await;

        let stream = engine
            .ws(r#"subscription { httpHeader(name: ["authorization", "other"]) }"#)
            .header("authorization", "super secret")
            .header("other", "not forwarded")
            .await
            .unwrap();

        let responses = stream.collect::<Vec<_>>().await;

        insta::assert_json_snapshot!(responses, @r#"
        [
          {
            "data": {
              "httpHeader": {
                "name": "authorization",
                "value": "super secret"
              }
            }
          },
          {
            "data": {
              "httpHeader": {
                "name": "other",
                "value": null
              }
            }
          }
        ]
        "#);
    });
}

#[test]
fn websocket_connection_init_payload() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FederatedProductsSchema)
            .with_websocket_urls()
            .build()
            .await;

        let mut stream = engine
            .ws("subscription { connectionInitPayload }")
            .init_payload(serde_json::json!({
                "authorization": "Bearer token",
                "somethingElse": true,
                "something": {
                    "nested": {
                        "level": 3,
                        "array": [1, 2, 3],
                    }
                }
            }))
            .await
            .unwrap();

        let first = stream.next().await.unwrap();
        assert!(stream.next().await.is_none());

        first
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "connectionInitPayload": {
          "authorization": "Bearer token",
          "somethingElse": true,
          "something": {
            "nested": {
              "level": 3,
              "array": [
                1,
                2,
                3
              ]
            }
          }
        }
      }
    }
    "#);
}

#[test]
fn websocket_connection_init_payload_forwarding_disabled() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FederatedProductsSchema)
            .with_toml_config(
                "
                [websockets]
                forward_connection_init_payload = false
            ",
            )
            .with_websocket_urls()
            .build()
            .await;

        let mut stream = engine
            .ws("subscription { connectionInitPayload }")
            .init_payload(serde_json::json!({
                "authorization": "Bearer token",
                "somethingElse": true,
                "something": {
                    "nested": {
                        "level": 3,
                        "array": [1, 2, 3],
                    }
                }
            }))
            .await
            .unwrap();

        let first = stream.next().await.unwrap();
        assert!(stream.next().await.is_none());

        first
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "connectionInitPayload": null
      }
    }
    "#);
}
