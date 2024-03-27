use gateway_v2::Gateway;
use graphql_mocks::{FakeFederationAccountsSchema, MockGraphQlServer};
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

    let response = runtime().block_on(super::execute(
        r"
        query ExampleQuery {
            me {
                username
                reviewCount
            }
        }
        ",
    ));

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
