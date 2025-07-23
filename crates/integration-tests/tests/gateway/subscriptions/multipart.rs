use graphql_mocks::{
    FederatedAccountsSchema, FederatedInventorySchema, FederatedProductsSchema, FederatedReviewsSchema,
};
use integration_tests::{gateway::Gateway, runtime};

#[test]
fn single_subgraph_subscription() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FederatedProductsSchema::default())
            .with_websocket_urls()
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
            .collect()
            .await
    });

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
}

#[test]
fn actual_federated_subscription() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_websocket_urls()
            .with_subgraph(FederatedAccountsSchema::default())
            .with_subgraph(FederatedProductsSchema::default())
            .with_subgraph(FederatedReviewsSchema::default())
            .with_subgraph(FederatedInventorySchema::default())
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
            .collect()
            .await
    });

    insta::assert_json_snapshot!(response.messages, @r###"
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
