use integration_tests::runtime;

#[test]
fn sibling_dependencies() {
    let response = runtime().block_on(super::execute(
        r"
        query ExampleQuery {
            me {
                id
                username
                cart {
                    products {
                        price
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
          "cart": {
            "products": [
              {
                "price": 22,
                "reviews": [
                  {
                    "author": {
                      "id": "1234",
                      "username": "Me"
                    },
                    "body": "Fedoras are one of the most fashionable hats around and can look great with a variety of outfits."
                  }
                ]
              },
              {
                "price": 55,
                "reviews": [
                  {
                    "author": null,
                    "body": "Beautiful Pink, my parrot loves it. Definitely recommend!"
                  }
                ]
              }
            ]
          }
        }
      }
    }
    "###);
}
