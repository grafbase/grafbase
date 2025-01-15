use engine::Engine;
use futures::StreamExt as _;
use graphql_mocks::FederatedProductsSchema;
use integration_tests::{federation::EngineExt as _, runtime};

#[test]
fn websockets_basic() {
    let (first, second) = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FederatedProductsSchema)
            .with_websocket_config()
            .build()
            .await;

        let mut stream = engine
            .execute_ws(None, "subscription { newProducts { upc } }")
            .await
            .unwrap();

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
fn websocket_connection_init_payload() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FederatedProductsSchema)
            .with_websocket_config()
            .build()
            .await;

        let mut stream = engine
            .execute_ws(
                Some(serde_json::json!({
                    "authorization": "Bearer token",
                    "somethingElse": true,
                    "something": {
                        "nested": {
                            "level": 3,
                            "array": [1, 2, 3],
                        }
                    }
                })),
                "subscription { connectionInitPayload }",
            )
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
            .with_custom_websocket_config(
                "
                [websockets]
                forward_connection_init_payload = false
            ",
            )
            .build()
            .await;

        let mut stream = engine
            .execute_ws(
                Some(serde_json::json!({
                    "authorization": "Bearer token",
                    "somethingElse": true,
                    "something": {
                        "nested": {
                            "level": 3,
                            "array": [1, 2, 3],
                        }
                    }
                })),
                "subscription { connectionInitPayload }",
            )
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
