use integration_tests::runtime;

#[test]
fn simple_requires() {
    let response = runtime().block_on(super::execute(
        r"
        query ExampleQuery {
            topProducts {
                name
                reviews {
                    author {
                        username
                        trustworthiness
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
            "reviews": [
              {
                "author": {
                  "username": "Me",
                  "trustworthiness": "REALLY_TRUSTED"
                }
              }
            ]
          },
          {
            "name": "Fedora",
            "reviews": [
              {
                "author": {
                  "username": "Me",
                  "trustworthiness": "REALLY_TRUSTED"
                }
              }
            ]
          },
          {
            "name": "Boater",
            "reviews": [
              {
                "author": {
                  "username": "User 7777",
                  "trustworthiness": "KINDA_TRUSTED"
                }
              }
            ]
          },
          {
            "name": "Jeans",
            "reviews": []
          },
          {
            "name": "Pink Jeans",
            "reviews": [
              {
                "author": null
              }
            ]
          }
        ]
      }
    }
    "###);
}

#[test]
fn requires_with_arguments() {
    let response = runtime().block_on(super::execute(
        r"
        query ExampleQuery {
            topProducts {
                name
                weight(unit: GRAM)
                shippingEstimate
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
            "weight": 100.0,
            "shippingEstimate": 1
          },
          {
            "name": "Fedora",
            "weight": 200.0,
            "shippingEstimate": 1
          },
          {
            "name": "Boater",
            "weight": 300.0,
            "shippingEstimate": 1
          },
          {
            "name": "Jeans",
            "weight": 400.0,
            "shippingEstimate": 3
          },
          {
            "name": "Pink Jeans",
            "weight": 500.0,
            "shippingEstimate": 3
          }
        ]
      }
    }
    "###);
}

#[test]
fn requires_with_fragment_spread() {
    let response = runtime().block_on(super::execute(
        r##"
        query ExampleQuery {
            shippingOptions {
                summary
                defaultDeliveryCompany {
                    id
                    name
                    companyType
                }
                modalities {
                    id
                    name
                    qualifiedName
                }
            }
        }
        "##,
    ));

    insta::assert_json_snapshot!(response, @"");
}
