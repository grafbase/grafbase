use std::net::SocketAddr;

use integration_tests::{runtime, Engine, EngineBuilder, ResponseExt};
use serde_json::json;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

#[test]
fn test_defer_on_matching_typecondition() {
    runtime().block_on(async {
        let mock_server = wiremock::MockServer::start().await;
        mock_pets(&mock_server, json!([doggo(json!("A PetstoreString"))])).await;

        let engine = build_engine(schema(mock_server.address())).await;

        insta::assert_json_snapshot!(
            engine
                .execute_stream(
                r#"
                    query {
                        petstore {
                            pets {
                                id
                                owner {
                                    __typename
                                    ... on PetstoreString @defer {
                                        data
                                    }
                                }
                            }
                        }
                    }
                "#,
                )
                .into_iter()
                .await
                .map(ResponseExt::into_value)
                .collect::<Vec<_>>(),
                @r###"
        [
          {
            "data": {
              "petstore": {
                "pets": [
                  {
                    "id": 123,
                    "owner": {
                      "__typename": "PetstoreString"
                    }
                  }
                ]
              }
            },
            "hasNext": true
          },
          {
            "data": {
              "data": "A PetstoreString"
            },
            "hasNext": false,
            "path": [
              "petstore",
              "pets",
              0,
              "owner"
            ]
          }
        ]
        "###);
    });
}

#[test]
fn test_defer_on_unmatching_typecondition() {
    runtime().block_on(async {
        let mock_server = wiremock::MockServer::start().await;
        mock_pets(&mock_server, json!([doggo(json!("A PetstoreString"))])).await;

        let engine = build_engine(schema(mock_server.address())).await;

        insta::assert_json_snapshot!(
            engine
                .execute_stream(
                r#"
                    query {
                        petstore {
                            pets {
                                id
                                owner {
                                    __typename
                                    ... on PetstorePerson @defer {
                                        id
                                    }
                                }
                            }
                        }
                    }
                "#,
                )
                .into_iter()
                .await
                .map(ResponseExt::into_value)
                .collect::<Vec<_>>(),
                @r###"
        [
          {
            "data": {
              "petstore": {
                "pets": [
                  {
                    "id": 123,
                    "owner": {
                      "__typename": "PetstoreString"
                    }
                  }
                ]
              }
            },
            "hasNext": false
          }
        ]
        "###);
    });
}

#[test]
fn test_defer_on_multiple_fragments_with_one_match() {
    runtime().block_on(async {
        let mock_server = wiremock::MockServer::start().await;
        mock_pets(&mock_server, json!([doggo(json!("A PetstoreString"))])).await;

        let engine = build_engine(schema(mock_server.address())).await;

        insta::assert_json_snapshot!(
            engine
                .execute_stream(
                r#"
                    query {
                        petstore {
                            pets {
                                id
                                owner {
                                    __typename
                                    ... on PetstorePerson @defer {
                                        id
                                    }
                                    ... on PetstoreString @defer {
                                        data
                                    }
                                }
                            }
                        }
                    }
                "#,
                )
                .into_iter()
                .await
                .map(ResponseExt::into_value)
                .collect::<Vec<_>>(),
                @r###"
        [
          {
            "data": {
              "petstore": {
                "pets": [
                  {
                    "id": 123,
                    "owner": {
                      "__typename": "PetstoreString"
                    }
                  }
                ]
              }
            },
            "hasNext": true
          },
          {
            "data": {
              "data": "A PetstoreString"
            },
            "hasNext": false,
            "path": [
              "petstore",
              "pets",
              0,
              "owner"
            ]
          }
        ]
        "###);
    });
}

#[test]
fn test_defer_with_typecondition_on_concrete_type() {
    runtime().block_on(async {
        let mock_server = wiremock::MockServer::start().await;
        mock_pets(&mock_server, json!([doggo(json!("A PetstoreString"))])).await;

        let engine = build_engine(schema(mock_server.address())).await;

        insta::assert_json_snapshot!(
            engine
                .execute_stream(
                r#"
                    query {
                        petstore {
                            ... on PetstoreQuery @defer {
                                pets {
                                    id
                                }
                            }
                        }
                    }
                "#,
                )
                .into_iter()
                .await
                .map(ResponseExt::into_value)
                .collect::<Vec<_>>(),
                @r###"
        [
          {
            "data": {
              "petstore": {}
            },
            "hasNext": true
          },
          {
            "data": {
              "pets": [
                {
                  "id": 123
                }
              ]
            },
            "hasNext": false,
            "path": [
              "petstore"
            ]
          }
        ]
        "###);
    });
}

async fn build_engine(schema: String) -> Engine {
    EngineBuilder::new(schema)
        .with_openapi_schema(
            "http://example.com/remote_unions.json",
            include_str!("../openapi/remote_union_spec.json"),
        )
        .build()
        .await
}

fn schema(address: &SocketAddr) -> String {
    format!(
        r#"
          extend schema
          @openapi(
            name: "petstore",
            url: "http://{address}",
            schema: "http://example.com/remote_unions.json"
          )
        "#
    )
}

async fn mock_pets(mock_server: &MockServer, pets: serde_json::Value) {
    Mock::given(method("GET"))
        .and(path("/pets"))
        .respond_with(ResponseTemplate::new(200).set_body_json(pets))
        .expect(1)
        .mount(mock_server)
        .await;
}

#[allow(clippy::needless_pass_by_value)]
fn doggo(owner: serde_json::Value) -> serde_json::Value {
    json!({ "id": 123, "owner": owner })
}
