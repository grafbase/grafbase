use gateway_v2::Gateway;
use graphql_mocks::{FakeFederationAccountsSchema, FakeFederationReviewsSchema, MockGraphQlServer};
use integration_tests::{federation::GatewayV2Ext, runtime};

#[test]
fn simple_override() {
    let response = runtime().block_on(async {
        let accounts = MockGraphQlServer::new(FakeFederationAccountsSchema).await;
        let engine = Gateway::builder()
            .with_schema("accounts", &accounts)
            .await
            .finish()
            .await;
        engine
            .execute(
                r"
                query ExampleQuery {
                    me {
                        username
                        reviewCount
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
          "username": "Me",
          "reviewCount": 0
        }
      }
    }
    "###);

    let response = runtime().block_on(async {
        let accounts = MockGraphQlServer::new(FakeFederationAccountsSchema).await;
        let reviews = MockGraphQlServer::new(FakeFederationReviewsSchema).await;
        let engine = Gateway::builder()
            .with_schema("accounts", &accounts)
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
                        username
                        reviewCount
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
          "username": "Me",
          "reviewCount": 2
        }
      }
    }
    "###);
}
