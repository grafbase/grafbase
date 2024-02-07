use integration_tests::federation::FederationGatewayWithoutIO;
use serde_json::json;

const SCHEMA: &str = include_str!("../../../data/federated-graph-schema.graphql");

#[test]
fn simple_error() {
    let bench = FederationGatewayWithoutIO::new(
        SCHEMA,
        r#"
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
        "#,
        &[
            json!({"data":{"me":{"id":"1234","username":"Me"}}}),
            // Missing author.id
            json!({"data":{"_entities":[{"__typename":"User","reviews":[{"body":"A highly effective form of birth control.","product":{"reviews":[{"author":{},"body":"A highly effective form of birth control."}]}},{"body":"Fedoras are one of the most fashionable hats around and can look great with a variety of outfits.","product":{"reviews":[{"author":{"id":"1234"},"body":"Fedoras are one of the most fashionable hats around and can look great with a variety of outfits."}]}}]}]}}),
            json!({"data":{"_entities":[{"__typename":"User","username":"Me"}]}}),
        ],
    );
    let response = integration_tests::runtime().block_on(bench.unchecked_execute());

    let json = serde_json::from_slice::<serde_json::Value>(&response.bytes).unwrap();
    insta::assert_json_snapshot!(json, @r###"
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
                    "author": null,
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
      },
      "errors": [
        {
          "message": "Upstream response error: Missing required field named 'id' at line 1 column 140",
          "locations": [
            {
              "line": 10,
              "column": 29
            }
          ],
          "path": [
            "me",
            "reviews",
            0,
            "product",
            "reviews",
            0,
            "author"
          ]
        }
      ]
    }
    "###);
}
