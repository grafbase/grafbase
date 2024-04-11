use futures::stream::StreamExt;

use gateway_v2::Gateway;
use graphql_mocks::{
    FakeFederationAccountsSchema, FakeFederationInventorySchema, FakeFederationProductsSchema,
    FakeFederationReviewsSchema, MockGraphQlServer,
};
use integration_tests::{federation::GatewayV2Ext, runtime};

#[test]
fn single_subgraph_subscription() {
    let response = runtime().block_on(async move {
        let products = MockGraphQlServer::new(FakeFederationProductsSchema).await;

        let engine = Gateway::builder()
            .with_schema("products", &products)
            .await
            .with_supergraph_config(indoc::formatdoc!(
                r#"
                    extend schema
                      @subgraph(name: "products", websocketUrl: "{}")
                "#,
                products.websocket_url(),
            ))
            .finish()
            .await;

        engine
            .execute(
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
            .into_stream()
            .collect::<Vec<_>>()
            .await
    });

    insta::assert_json_snapshot!(response, @r###"
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
        let accounts = MockGraphQlServer::new(FakeFederationAccountsSchema).await;
        let products = MockGraphQlServer::new(FakeFederationProductsSchema).await;
        let reviews = MockGraphQlServer::new(FakeFederationReviewsSchema).await;
        let inventory = MockGraphQlServer::new(FakeFederationInventorySchema).await;

        let engine = Gateway::builder()
            .with_schema("accounts", &accounts)
            .await
            .with_schema("products", &products)
            .await
            .with_schema("reviews", &reviews)
            .await
            .with_schema("inventory", &inventory)
            .await
            .with_supergraph_config(indoc::formatdoc!(
                r#"
                    extend schema
                      @subgraph(name: "accounts", websocketUrl: "{}")
                      @subgraph(name: "products", websocketUrl: "{}")
                      @subgraph(name: "reviews", websocketUrl: "{}")
                      @subgraph(name: "inventory", websocketUrl: "{}")
                "#,
                accounts.websocket_url(),
                products.websocket_url(),
                reviews.websocket_url(),
                inventory.websocket_url(),
            ))
            .finish()
            .await;

        engine
            .execute(
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
            .into_stream()
            .collect::<Vec<_>>()
            .await
    });

    insta::assert_json_snapshot!(response, @r###"
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
