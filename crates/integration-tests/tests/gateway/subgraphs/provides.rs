use integration_tests::runtime;

#[test]
fn provides_with_fragment_spread() {
    let response = runtime().block_on(super::execute(
        r##"
        query ExampleQuery {
            shippingOptions {
                defaultDeliveryCompany {
                    id
                    name
                    companyType
                }
                seller {
                    ... on BusinessAccount {
                        id
                        email
                        joinedTimestamp
                    }
                    ... on User {
                        id
                        username
                        reviewCount
                    }
                }
            }
        }
        "##,
    ));

    // Here we expect the email to be "email@from-shipping-subgraph.net" because that is the value in the shipping subgraph.

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "shippingOptions": {
          "defaultDeliveryCompany": {
            "id": "1",
            "name": "Planet Express",
            "companyType": "GmbH"
          },
          "seller": {
            "id": "ba_2",
            "email": "email@from-shipping-subgraph.net",
            "joinedTimestamp": 1234567890
          }
        }
      }
    }
    "#);
}
