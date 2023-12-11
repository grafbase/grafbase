use engine_v2::Engine;
use integration_tests::{
    federation::EngineV2Ext,
    mocks::graphql::{FakeFederationAccountsSchema, FakeFederationProductsSchema, FakeFederationReviewsSchema},
    runtime, MockGraphQlServer,
};

#[test]
fn simple_key() {
    let response = runtime().block_on(async move {
        let accounts = MockGraphQlServer::new(FakeFederationAccountsSchema).await;
        let products = MockGraphQlServer::new(FakeFederationProductsSchema).await;
        let reviews = MockGraphQlServer::new(FakeFederationReviewsSchema).await;

        let engine = Engine::build()
            .with_schema("accounts", &accounts)
            .await
            .with_schema("products", &products)
            .await
            .with_schema("reviews", &reviews)
            .await
            .finish()
            .await;

        engine
            .execute(
                r"
                query ExampleQuery {
                    me {
                        id
                        username
                        reviews {
                            body
                            product {
                                reviews {
                                    author {
                                        id
                                        username
                                    }
                                    body
                                }
                            }
                        }
                    }
                }
                ",
            )
            .await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "me": {
          "id": "1234",
          "username": "Me",
          "reviews": [
            {
              "body": "A highly effective form of birth control.",
              "product": {
                "reviews": [
                  {
                    "author": {
                      "id": "1234",
                      "username": "Me"
                    },
                    "body": "A highly effective form of birth control."
                  }
                ]
              }
            },
            {
              "body": "Fedoras are one of the most fashionable hats around and can look great with a variety of outfits.",
              "product": {
                "reviews": [
                  {
                    "author": {
                      "id": "1234",
                      "username": "Me"
                    },
                    "body": "Fedoras are one of the most fashionable hats around and can look great with a variety of outfits."
                  }
                ]
              }
            }
          ]
        }
      }
    }
    "###);
}
