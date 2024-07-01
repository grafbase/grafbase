use integration_tests::federation::DeterministicEngine;
use serde_json::json;

const SCHEMA: &str = include_str!("../../../data/federated-graph-schema.graphql");

#[test]
fn subgraph_no_response() {
    let response = integration_tests::runtime().block_on(async {
        DeterministicEngine::new(
            SCHEMA,
            r#"
        query ExampleQuery {
            me {
                id
            }
        }
        "#,
            &[json!(null)],
        )
        .await
        .execute()
        .await
    });
    insta::assert_json_snapshot!(response, @r###"
    {
      "data": null,
      "errors": [
        {
          "message": "Deserialization error: invalid type: null, expected a valid GraphQL response at line 1 column 4",
          "path": [
            "me"
          ]
        }
      ]
    }
    "###);

    let response = integration_tests::runtime().block_on(async {
        DeterministicEngine::new(
            SCHEMA,
            r#"
        query ExampleQuery {
            me {
                id
            }
        }
        "#,
            &[json!({})],
        )
        .await
        .execute()
        .await
    });
    insta::assert_json_snapshot!(response, @r###"
    {
      "data": null,
      "errors": [
        {
          "message": "Missing data from subgraph",
          "path": [
            "me"
          ]
        }
      ]
    }
    "###);
}

#[test]
fn request_error() {
    let response = integration_tests::runtime().block_on(async {
        DeterministicEngine::new(
            SCHEMA,
            r#"
        query ExampleQuery {
            _typean_
        }
        "#,
            &[json!({})],
        )
        .await
        .execute()
        .await
    });
    insta::assert_json_snapshot!(response, @r###"
    {
      "errors": [
        {
          "message": "Query does not have a field named '_typean_'",
          "locations": [
            {
              "line": 3,
              "column": 13
            }
          ]
        }
      ]
    }
    "###);
}

#[test]
fn sugraph_request_error() {
    let response = integration_tests::runtime().block_on(async {
        DeterministicEngine::new(
            SCHEMA,
            r#"
        query ExampleQuery {
            me {
                id
            }
        }
        "#,
            &[json!({"errors": [{"message": "failed!"}]})],
        )
        .await
        .execute()
        .await
    });
    insta::assert_json_snapshot!(response, @r###"
    {
      "data": null,
      "errors": [
        {
          "message": "Upstream error: failed!"
        }
      ]
    }
    "###);
}

#[test]
fn invalid_response_for_nullable_field() {
    let response = integration_tests::runtime().block_on(async {
        DeterministicEngine::new(
            SCHEMA,
            r#"
        query ExampleQuery {
            name
        }
        "#,
            &[json!({})],
        )
        .await
        .execute()
        .await
    });
    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "name": null
      },
      "errors": [
        {
          "message": "Missing data from subgraph",
          "path": [
            "name"
          ]
        }
      ]
    }
    "###);
}

#[test]
fn subgraph_field_error() {
    let response = integration_tests::runtime().block_on(async {
        DeterministicEngine::new(
            SCHEMA,
            r#"
        query ExampleQuery {
            me {
                id
            }
        }
        "#,
            &[json!({"data": null, "errors": [{"message": "failed!", "path": ["me", "id"]}]})],
        )
        .await
        .execute()
        .await
    });
    insta::assert_json_snapshot!(response, @r###"
    {
      "data": null,
      "errors": [
        {
          "message": "Upstream error: failed!",
          "path": [
            "me",
            "id"
          ]
        }
      ]
    }
    "###);
}

#[test]
fn simple_error() {
    let response = integration_tests::runtime().block_on(async { DeterministicEngine::new(
        SCHEMA,
        indoc::indoc! {"
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
            "},
            &[
                json!({"data":{"me":{"id":"1234","username":"Me"}}}),
                // Missing author.id
                json!({"data":{"_entities":[
                {"__typename":"User",
                 "reviews":[
                    {"body":"A highly effective form of birth control.","product":{"reviews":[{"author":{},"body":"A highly effective form of birth control."}]}},
                    {"body":"Fedoras are one of the most fashionable hats around and can look great with a variety of outfits.","product":{"reviews":[{"author":{"id":"1234"},"body":"Fedoras are one of the most fashionable hats around and can look great with a variety of outfits."}]}}
                ]}]}}),
                json!({"data":{"_entities":[{"__typename":"User","username":"Me"}]}}),
            ],
        ).await.execute().await
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
          "message": "Error decoding response from upstream: Missing required field named 'id' at line 1 column 140",
          "locations": [
            {
              "line": 9,
              "column": 21
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

#[test]
fn null_entity_with_error() {
    let response = integration_tests::runtime().block_on(async { DeterministicEngine::new(
        SCHEMA,
        r#"
            query ExampleQuery {
                me {
                    id
                    username
                    reviews {
                        body
                    }
                }
            }
            "#,
            &[
                json!({"data":{"me":{"id":"1234","username":"Me"}}}),
                json!({"data":{"_entities":[null]}, "errors": [{"message":"I'm broken!", "path": ["_entities", 0, "reviews", 0, "body"]}]}),
            ],
        ).await.execute().await
    });
    insta::assert_json_snapshot!(response, @r###"
    {
      "data": null,
      "errors": [
        {
          "message": "Upstream error: I'm broken!",
          "path": [
            "me",
            "reviews",
            0,
            "body"
          ]
        }
      ]
    }
    "###);
}
