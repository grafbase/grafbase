use integration_tests::runtime;

#[test]
fn interface_object() {
    let response = runtime().block_on(super::execute(
        r"
        query ExampleQuery {
            topProducts {
                name
                availableShippingService {
                    __typename
                    name
                    reviews {
                        body
                    }
                }
            }
        }
        ",
    ));

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "topProducts": [
          {
            "name": "Trilby",
            "availableShippingService": [
              {
                "__typename": "DeliveryCompany",
                "name": "Planet Express",
                "reviews": [
                  {
                    "body": "Not as good as Mom's Friendly Delivery Company"
                  }
                ]
              },
              {
                "__typename": "HomingPigeon",
                "name": "Cher Ami",
                "reviews": [
                  {
                    "body": "Saved my life in the war"
                  }
                ]
              }
            ]
          },
          {
            "name": "Fedora",
            "availableShippingService": [
              {
                "__typename": "DeliveryCompany",
                "name": "Planet Express",
                "reviews": [
                  {
                    "body": "Not as good as Mom's Friendly Delivery Company"
                  }
                ]
              }
            ]
          },
          {
            "name": "Boater",
            "availableShippingService": [
              {
                "__typename": "DeliveryCompany",
                "name": "Planet Express",
                "reviews": [
                  {
                    "body": "Not as good as Mom's Friendly Delivery Company"
                  }
                ]
              }
            ]
          },
          {
            "name": "Jeans",
            "availableShippingService": [
              {
                "__typename": "DeliveryCompany",
                "name": "Planet Express",
                "reviews": [
                  {
                    "body": "Not as good as Mom's Friendly Delivery Company"
                  }
                ]
              }
            ]
          },
          {
            "name": "Pink Jeans",
            "availableShippingService": [
              {
                "__typename": "DeliveryCompany",
                "name": "Planet Express",
                "reviews": [
                  {
                    "body": "Not as good as Mom's Friendly Delivery Company"
                  }
                ]
              }
            ]
          }
        ]
      }
    }
    "###);
}
