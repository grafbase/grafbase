use engine::Engine;
use graphql_mocks::{
    FederatedAccountsSchema, FederatedInventorySchema, FederatedProductsSchema, FederatedReviewsSchema,
};
use integration_tests::{federation::EngineExt, runtime};

#[test]
fn single_subgraph_subscription() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FederatedProductsSchema)
            .with_websocket_config()
            .build()
            .await;

        engine
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
            .into_sse_stream()
            .await
    });

    insta::assert_json_snapshot!(response.collected_body, @r###"
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
}

#[test]
fn request_error() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FederatedProductsSchema)
            .with_websocket_config()
            .build()
            .await;

        engine
            .post(
                r"
                subscription {
                    unknown
                }
                ",
            )
            .into_sse_stream()
            .await
    });

    insta::assert_json_snapshot!(response.collected_body, @r###"
    [
      {
        "errors": [
          {
            "message": "Subscription does not have a field named 'unknown'",
            "locations": [
              {
                "line": 3,
                "column": 21
              }
            ],
            "extensions": {
              "code": "OPERATION_VALIDATION_ERROR"
            }
          }
        ]
      }
    ]
    "###);
    assert_eq!(response.status, http::StatusCode::OK);
}

#[test]
fn actual_federated_subscription() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_websocket_config()
            .with_subgraph(FederatedAccountsSchema)
            .with_subgraph(FederatedProductsSchema)
            .with_subgraph(FederatedReviewsSchema)
            .with_subgraph(FederatedInventorySchema)
            .build()
            .await;

        engine
            .post(
                r"
                subscription {
                    newProducts {
                        upc
                        name
                        reviews {
                            author {
                                username
                            }
                            body
                        }
                    }
                }
                ",
            )
            .into_sse_stream()
            .await
    });

    insta::assert_json_snapshot!(response.collected_body, @r###"
    [
      {
        "data": {
          "newProducts": {
            "upc": "top-4",
            "name": "Jeans",
            "reviews": []
          }
        }
      },
      {
        "data": {
          "newProducts": {
            "upc": "top-5",
            "name": "Pink Jeans",
            "reviews": [
              {
                "author": null,
                "body": "Beautiful Pink, my parrot loves it. Definitely recommend!"
              }
            ]
          }
        }
      }
    ]
    "###);
}
