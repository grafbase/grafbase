use engine_v2::Engine;
use graphql_mocks::{
    FederatedAccountsSchema, FederatedInventorySchema, FederatedProductsSchema, FederatedReviewsSchema,
};
use integration_tests::{federation::EngineV2Ext, runtime};

#[test]
fn single_subgraph_subscription() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FederatedProductsSchema)
            .with_sdl_websocket_config()
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
            .into_multipart_stream()
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
fn actual_federated_subscription() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FederatedAccountsSchema)
            .with_subgraph(FederatedProductsSchema)
            .with_subgraph(FederatedReviewsSchema)
            .with_subgraph(FederatedInventorySchema)
            .with_sdl_websocket_config()
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
            .into_multipart_stream()
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
