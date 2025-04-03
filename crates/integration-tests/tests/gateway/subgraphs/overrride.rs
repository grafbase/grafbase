use graphql_mocks::FederatedAccountsSchema;
use integration_tests::{gateway::Gateway, runtime};

#[test]
fn simple_override() {
    let response = runtime().block_on(async {
        let engine = Gateway::builder().with_subgraph(FederatedAccountsSchema).build().await;
        engine
            .post(
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
