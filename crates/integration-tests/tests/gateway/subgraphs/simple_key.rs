use integration_tests::runtime;

#[test]
fn simple_key_basic() {
    let response = runtime().block_on(super::execute(
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
    ));

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

#[test]
fn simple_key_with_missing_required_fields() {
    let response = runtime().block_on(super::execute(
        r"
        query ExampleQuery {
            me {
                username
                reviews {
                    body
                    product {
                        name
                        price
                        reviews {
                            author {
                                username
                            }
                            body
                        }
                    }
                }
            }
        }
        ",
    ));

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "me": {
          "username": "Me",
          "reviews": [
            {
              "body": "A highly effective form of birth control.",
              "product": {
                "name": "Trilby",
                "price": 10,
                "reviews": [
                  {
                    "author": {
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
                "name": "Fedora",
                "price": 20,
                "reviews": [
                  {
                    "author": {
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

#[test]
fn simple_key_with_simple_fragments() {
    let response = runtime().block_on(super::execute(
        r"
        query ExampleQuery {
            me {
                ...Username
                username
                reviews {
                    ...ReviewProducts
                    body
                    product {
                        reviews {
                            author {
                                ... on User {
                                    username
                                }
                            }
                        }
                    }
                }
            }
        }

        fragment Username on User {
            username
        }

        fragment ReviewProducts on Review {
            product {
                name
                price
            }
        }
        ",
    ));

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "me": {
          "username": "Me",
          "reviews": [
            {
              "product": {
                "name": "Trilby",
                "price": 10,
                "reviews": [
                  {
                    "author": {
                      "username": "Me"
                    }
                  }
                ]
              },
              "body": "A highly effective form of birth control."
            },
            {
              "product": {
                "name": "Fedora",
                "price": 20,
                "reviews": [
                  {
                    "author": {
                      "username": "Me"
                    }
                  }
                ]
              },
              "body": "Fedoras are one of the most fashionable hats around and can look great with a variety of outfits."
            }
          ]
        }
      }
    }
    "###);
}

#[test]
fn simple_key_with_inexistent_entities() {
    let response = runtime().block_on(super::execute(
        r"
        query ExampleQuery {
            topProducts {
                name
                price
                reviews {
                    author {
                        username
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
            "price": 11,
            "reviews": [
              {
                "author": {
                  "username": "Me"
                }
              }
            ]
          },
          {
            "name": "Fedora",
            "price": 22,
            "reviews": [
              {
                "author": {
                  "username": "Me"
                }
              }
            ]
          },
          {
            "name": "Boater",
            "price": 33,
            "reviews": [
              {
                "author": {
                  "username": "User 7777"
                }
              }
            ]
          },
          {
            "name": "Jeans",
            "price": 44,
            "reviews": []
          },
          {
            "name": "Pink Jeans",
            "price": 55,
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
