mod type_conditions;

use std::net::SocketAddr;

use integration_tests::{runtime, Engine, EngineBuilder, ResponseExt};
use serde_json::json;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

#[test]
fn simple_defer_test() {
    runtime().block_on(async {
        let mock_server = wiremock::MockServer::start().await;
        let engine = build_engine(petstore_schema(mock_server.address())).await;

        mock_doggo(&mock_server, 123, "Immediate Doggo").await;
        mock_doggo(&mock_server, 456, "Deferred Doggo").await;

        insta::assert_json_snapshot!(
            engine
                .execute_stream(
                r#"
                    query {
                        petstore {
                            pet(petId: 123) {
                                id
                                name
                            }
                            ... @defer {
                                deferredPet: pet(petId: 456) {
                                    id
                                    name
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
                "pet": {
                  "id": 123,
                  "name": "Immediate Doggo"
                }
              }
            }
          },
          {
            "data": {
              "deferredPet": {
                "id": 456,
                "name": "Deferred Doggo"
              }
            },
            "hasNext": false,
            "path": [
              "petstore"
            ]
          }
        ]
        "###
        );
    });
}

#[test]
fn defer_on_non_streaming_request_doesnt_defer() {
    runtime().block_on(async {
        let mock_server = wiremock::MockServer::start().await;
        let engine = build_engine(petstore_schema(mock_server.address())).await;

        mock_doggo(&mock_server, 123, "Immediate Doggo").await;
        mock_doggo(&mock_server, 456, "Deferred Doggo").await;

        insta::assert_json_snapshot!(
            engine
                .execute(
                r#"
                    query {
                        petstore {
                            pet(petId: 123) {
                                id
                                name
                            }
                            ... @defer {
                                deferredPet: pet(petId: 456) {
                                    id
                                    name
                                }
                            }
                        }
                    }
                "#,
                )
                .await
                .into_value(),
            @r###"
        {
          "data": {
            "petstore": {
              "deferredPet": {
                "id": 456,
                "name": "Deferred Doggo"
              },
              "pet": {
                "id": 123,
                "name": "Immediate Doggo"
              }
            }
          }
        }
        "###
        );
    });
}

#[test]
fn test_defer_on_field_rejected() {
    runtime().block_on(async {
        let mock_server = wiremock::MockServer::start().await;
        let engine = build_engine(petstore_schema(mock_server.address())).await;

        insta::assert_json_snapshot!(engine
            .execute(
                r#"
                query {
                    petstore @defer {
                        pet(petId: 123) {
                            id
                            name
                        }
                    }
                }
            "#
            )
            .await
            .into_value(), @r###"
        {
          "data": null,
          "errors": [
            {
              "locations": [
                {
                  "column": 30,
                  "line": 3
                }
              ],
              "message": "Directive \"defer\" may not be used on \"FIELD\""
            }
          ]
        }
        "###);
    });
}

#[test]
fn test_defer_on_named_fragment() {
    runtime().block_on(async {
        let mock_server = wiremock::MockServer::start().await;
        let engine = build_engine(petstore_schema(mock_server.address())).await;

        mock_doggo(&mock_server, 123, "Immediate Doggo").await;
        mock_doggo(&mock_server, 456, "Deferred Doggo").await;

        insta::assert_json_snapshot!(engine
            .execute_stream(
                r#"
                    query {
                        petstore {
                            pet(petId: 123) {
                                id
                                name
                            }
                            ...DeferredFragment @defer
                        }
                    }

                    fragment DeferredFragment on PetstoreQuery {
                        deferredPet: pet(petId: 456) {
                            id
                            name
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
                "pet": {
                  "id": 123,
                  "name": "Immediate Doggo"
                }
              }
            }
          },
          {
            "data": {
              "deferredPet": {
                "id": 456,
                "name": "Deferred Doggo"
              }
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

#[test]
fn test_nested_defers() {
    runtime().block_on(async {
        let mock_server = wiremock::MockServer::start().await;
        let engine = build_engine(petstore_schema(mock_server.address())).await;

        mock_doggo(&mock_server, 123, "First Deferred Doggo").await;
        mock_doggo(&mock_server, 456, "Second Deferred Doggo").await;

        insta::assert_json_snapshot!(
            engine
                .execute_stream(
                r#"
                    query {
                        petstore {
                          ... @defer {
                            firstPet: pet(petId: 123) {
                                id
                                name
                            }
                            ... @defer {
                                secondPet: pet(petId: 456) {
                                    id
                                    name
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
              "petstore": {}
            }
          },
          {
            "data": {
              "firstPet": {
                "id": 123,
                "name": "First Deferred Doggo"
              }
            },
            "hasNext": true,
            "path": [
              "petstore"
            ]
          },
          {
            "data": {
              "secondPet": {
                "id": 456,
                "name": "Second Deferred Doggo"
              }
            },
            "hasNext": false,
            "path": [
              "petstore"
            ]
          }
        ]
        "###
        );
    });
}

#[test]
fn test_defer_with_errors() {
    runtime().block_on(async {
        let mock_server = wiremock::MockServer::start().await;
        let engine = build_engine(petstore_schema(mock_server.address())).await;

        // We're specifically not registering any mock pets so both
        // the fields in the query below should error

        insta::assert_json_snapshot!(
            engine
                .execute_stream(
                r#"
                    query {
                        petstore {
                            pet(petId: 123) {
                                id
                                name
                            }
                            ... @defer {
                                deferredPet: pet(petId: 456) {
                                    id
                                    name
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
                "pet": null
              }
            },
            "errors": [
              {
                "locations": [
                  {
                    "column": 29,
                    "line": 4
                  }
                ],
                "message": "Received an unexpected status from the downstream server: 404 Not Found",
                "path": [
                  "petstore",
                  "pet"
                ]
              }
            ]
          },
          {
            "data": {
              "deferredPet": null
            },
            "errors": [
              {
                "locations": [
                  {
                    "column": 33,
                    "line": 9
                  }
                ],
                "message": "Received an unexpected status from the downstream server: 404 Not Found",
                "path": [
                  "petstore",
                  "deferredPet"
                ]
              }
            ],
            "hasNext": false,
            "path": [
              "petstore"
            ]
          }
        ]
        "###
        );
    });
}

#[test]
fn test_defer_at_root() {
    runtime().block_on(async {
        let mock_server = wiremock::MockServer::start().await;
        let engine = build_engine(petstore_schema(mock_server.address())).await;

        mock_doggo(&mock_server, 123, "Immediate Doggo").await;
        mock_doggo(&mock_server, 456, "Deferred Doggo").await;

        insta::assert_json_snapshot!(
            engine
                .execute_stream(
                r#"
                    query {
                        petstore {
                            pet(petId: 123) {
                                id
                                name
                            }
						}
						... @defer {
							petstore {
                                deferredPet: pet(petId: 456) {
                                    id
                                    name
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
                "pet": {
                  "id": 123,
                  "name": "Immediate Doggo"
                }
              }
            }
          },
          {
            "data": {
              "petstore": {
                "deferredPet": {
                  "id": 456,
                  "name": "Deferred Doggo"
                }
              }
            },
            "hasNext": false,
            "path": []
          }
        ]
        "###
        );
    });
}

#[test]
fn test_defer_with_labels() {
    runtime().block_on(async {
        let mock_server = wiremock::MockServer::start().await;
        let engine = build_engine(petstore_schema(mock_server.address())).await;

        mock_doggo(&mock_server, 123, "First Deferred Doggo").await;
        mock_doggo(&mock_server, 456, "Second Deferred Doggo").await;

        insta::assert_json_snapshot!(
            engine
                .execute_stream(
                r#"
                    query {
                        petstore {
                          ... @defer(label: "outer") {
                            firstPet: pet(petId: 123) {
                                id
                                name
                            }
                            ... @defer(label: "inner") {
                                secondPet: pet(petId: 456) {
                                    id
                                    name
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
              "petstore": {}
            }
          },
          {
            "data": {
              "firstPet": {
                "id": 123,
                "name": "First Deferred Doggo"
              }
            },
            "hasNext": true,
            "label": "outer",
            "path": [
              "petstore"
            ]
          },
          {
            "data": {
              "secondPet": {
                "id": 456,
                "name": "Second Deferred Doggo"
              }
            },
            "hasNext": false,
            "label": "inner",
            "path": [
              "petstore"
            ]
          }
        ]
        "###
        );
    });
}

#[test]
fn test_defer_with_if_true() {
    runtime().block_on(async {
        let mock_server = wiremock::MockServer::start().await;
        let engine = build_engine(petstore_schema(mock_server.address())).await;

        mock_doggo(&mock_server, 123, "Immediate Doggo").await;
        mock_doggo(&mock_server, 456, "Deferred Doggo").await;

        insta::assert_json_snapshot!(
            engine
                .execute_stream(
                r#"
                    query {
                        petstore {
                            pet(petId: 123) {
                                id
                                name
                            }
                            ... @defer(if: true) {
                                deferredPet: pet(petId: 456) {
                                    id
                                    name
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
                "pet": {
                  "id": 123,
                  "name": "Immediate Doggo"
                }
              }
            }
          },
          {
            "data": {
              "deferredPet": {
                "id": 456,
                "name": "Deferred Doggo"
              }
            },
            "hasNext": false,
            "path": [
              "petstore"
            ]
          }
        ]
        "###
        );
    });
}

#[test]
fn test_defer_with_if_false() {
    runtime().block_on(async {
        let mock_server = wiremock::MockServer::start().await;
        let engine = build_engine(petstore_schema(mock_server.address())).await;

        mock_doggo(&mock_server, 123, "Immediate Doggo").await;
        mock_doggo(&mock_server, 456, "Deferred Doggo").await;

        insta::assert_json_snapshot!(
            engine
                .execute_stream(
                r#"
                    query {
                        petstore {
                            pet(petId: 123) {
                                id
                                name
                            }
                            ... @defer(if: false) {
                                notActuallyDeferredPet: pet(petId: 456) {
                                    id
                                    name
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
                "notActuallyDeferredPet": {
                  "id": 456,
                  "name": "Deferred Doggo"
                },
                "pet": {
                  "id": 123,
                  "name": "Immediate Doggo"
                }
              }
            }
          }
        ]
        "###
        );
    });
}

#[test]
fn test_invalid_defer_parameters() {
    runtime().block_on(async {
        let mock_server = wiremock::MockServer::start().await;
        let engine = build_engine(petstore_schema(mock_server.address())).await;

        insta::assert_json_snapshot!(
            engine
                .execute_stream(
                r#"
                    query {
                        petstore {
                            ... @defer(if: "hello") {
                                pet(petId: 456) {
                                    id
                                    name
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
            "data": null,
            "errors": [
              {
                "locations": [
                  {
                    "column": 40,
                    "line": 4
                  }
                ],
                "message": "Invalid value for argument \"if\", expected type \"Boolean\""
              }
            ]
          }
        ]
        "###
        );
    });
}

async fn build_engine(schema: String) -> Engine {
    EngineBuilder::new(schema)
        .with_openapi_schema(
            "http://example.com/petstore.json",
            include_str!("../openapi/petstore.json"),
        )
        .build()
        .await
}

fn petstore_schema(address: &SocketAddr) -> String {
    format!(
        r#"
          extend schema
          @openapi(
            name: "petstore",
            url: "http://{address}",
            schema: "http://example.com/petstore.json",
          )
        "#
    )
}

async fn mock_doggo(mock_server: &MockServer, id: u32, name: &str) {
    Mock::given(method("GET"))
        .and(path(format!("/pet/{id}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(doggo(id, name)))
        .mount(mock_server)
        .await;
}

fn doggo(id: u32, name: &str) -> serde_json::Value {
    json!({
        "id": id,
        "name": name,
        "category": {
            "id": 1,
            "name": "Dogs"
        },
        "photoUrls": [
            "string"
        ],
        "tags": [
            {
            "id": 0,
            "name": "string"
            }
        ],
        "status": "available"
    })
}
