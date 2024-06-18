mod type_conditions;

use std::net::SocketAddr;

use integration_tests::{runtime, udfs::RustUdfs, Engine, EngineBuilder, ResponseExt};
use runtime::udf::{CustomResolverRequestPayload, UdfResponse};
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
                r"
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
                ",
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
            },
            "hasNext": true
          },
          {
            "data": {
              "deferredPet": {
                "id": 456,
                "name": "Deferred Doggo"
              }
            },
            "path": [
              "petstore"
            ],
            "hasNext": false
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
                r"
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
                ",
                )
                .await
                .into_value(),
            @r###"
        {
          "data": {
            "petstore": {
              "pet": {
                "id": 123,
                "name": "Immediate Doggo"
              },
              "deferredPet": {
                "id": 456,
                "name": "Deferred Doggo"
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
                r"
                query {
                    petstore @defer {
                        pet(petId: 123) {
                            id
                            name
                        }
                    }
                }
            "
            )
            .await
            .into_value(), @r###"
        {
          "errors": [
            {
              "message": "Directive \"defer\" may not be used on \"Field\"",
              "locations": [
                {
                  "line": 3,
                  "column": 30
                }
              ]
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
                r"
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
                ",
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
            },
            "hasNext": true
          },
          {
            "data": {
              "deferredPet": {
                "id": 456,
                "name": "Deferred Doggo"
              }
            },
            "path": [
              "petstore"
            ],
            "hasNext": false
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
                r"
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
                ",
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
              "firstPet": {
                "id": 123,
                "name": "First Deferred Doggo"
              }
            },
            "path": [
              "petstore"
            ],
            "hasNext": true
          },
          {
            "data": {
              "secondPet": {
                "id": 456,
                "name": "Second Deferred Doggo"
              }
            },
            "path": [
              "petstore"
            ],
            "hasNext": false
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
                r"
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
                ",
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
                "message": "Received an unexpected status from the downstream server: 404 Not Found",
                "locations": [
                  {
                    "line": 4,
                    "column": 29
                  }
                ],
                "path": [
                  "petstore",
                  "pet"
                ]
              }
            ],
            "hasNext": true
          },
          {
            "data": {
              "deferredPet": null
            },
            "path": [
              "petstore"
            ],
            "hasNext": false,
            "errors": [
              {
                "message": "Received an unexpected status from the downstream server: 404 Not Found",
                "locations": [
                  {
                    "line": 9,
                    "column": 33
                  }
                ],
                "path": [
                  "petstore",
                  "deferredPet"
                ]
              }
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
                r"
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
                ",
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
            },
            "hasNext": true
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
            "path": [],
            "hasNext": false
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
            },
            "hasNext": true
          },
          {
            "data": {
              "firstPet": {
                "id": 123,
                "name": "First Deferred Doggo"
              }
            },
            "path": [
              "petstore"
            ],
            "hasNext": true,
            "label": "outer"
          },
          {
            "data": {
              "secondPet": {
                "id": 456,
                "name": "Second Deferred Doggo"
              }
            },
            "path": [
              "petstore"
            ],
            "hasNext": false,
            "label": "inner"
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
                r"
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
                ",
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
            },
            "hasNext": true
          },
          {
            "data": {
              "deferredPet": {
                "id": 456,
                "name": "Deferred Doggo"
              }
            },
            "path": [
              "petstore"
            ],
            "hasNext": false
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
                r"
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
                ",
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
                },
                "notActuallyDeferredPet": {
                  "id": 456,
                  "name": "Deferred Doggo"
                }
              }
            },
            "hasNext": false
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
            "errors": [
              {
                "message": "Invalid value for argument \"if\", expected type \"Boolean\"",
                "locations": [
                  {
                    "line": 4,
                    "column": 40
                  }
                ]
              }
            ],
            "hasNext": false
          }
        ]
        "###
        );
    });
}

#[test]
fn defer_a_custom_resolver() {
    // Tests that custom resolvers can live inside custom resolvers
    runtime().block_on(async {
        let schema = r#"
            type Query {
                list: [ListItem]! @resolver(name: "list")
            }

            type ListItem {
                item: Int! @resolver(name: "item")
            }
        "#;
        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(
                RustUdfs::new()
                    .resolver("list", UdfResponse::Success(json!([{"id": 1}, {"id": 2}])))
                    .resolver("item", |payload: CustomResolverRequestPayload| {
                        Ok(UdfResponse::Success(payload.parent.unwrap()["id"].clone()))
                    }),
            )
            .build()
            .await;

        insta::assert_json_snapshot!(
            engine.execute_stream("query { list { ... @defer { item } } }")
                .into_iter()
                .await
                .map(ResponseExt::into_value)
                .collect::<Vec<_>>(),
            @r###"
        [
          {
            "data": {
              "list": [
                {},
                {}
              ]
            },
            "hasNext": true
          },
          {
            "data": {
              "item": 1
            },
            "path": [
              "list",
              0
            ],
            "hasNext": true
          },
          {
            "data": {
              "item": 2
            },
            "path": [
              "list",
              1
            ],
            "hasNext": false
          }
        ]
        "###
        );
    });
}

#[test]
fn defer_a_custom_resolver_that_errors() {
    // Tests that custom resolvers can live inside custom resolvers
    runtime().block_on(async {
        let schema = r#"
            type Query {
                list: [ObjectWithErrors]! @resolver(name: "list")
            }

            type ObjectWithErrors {
                item: Int! @resolver(name: "item")
            }
        "#;
        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(
                RustUdfs::new()
                    .resolver("list", UdfResponse::Success(json!([{"id": 1}, {"id": 2}])))
                    .resolver(
                        "item",
                        UdfResponse::GraphQLError {
                            message: "I'm afraid I can't do that Dave".into(),
                            extensions: None,
                        },
                    ),
            )
            .build()
            .await;

        insta::assert_json_snapshot!(
            engine.execute_stream("query { list { ... @defer { item } } }")
                .into_iter()
                .await
                .map(ResponseExt::into_value)
                .collect::<Vec<_>>(),
            @r###"
        [
          {
            "data": {
              "list": [
                {},
                {}
              ]
            },
            "hasNext": true
          },
          {
            "data": null,
            "path": [
              "list",
              0
            ],
            "hasNext": true,
            "errors": [
              {
                "message": "I'm afraid I can't do that Dave",
                "locations": [
                  {
                    "line": 1,
                    "column": 29
                  }
                ],
                "path": [
                  "list",
                  0,
                  "item"
                ]
              }
            ]
          },
          {
            "data": null,
            "path": [
              "list",
              1
            ],
            "hasNext": false,
            "errors": [
              {
                "message": "I'm afraid I can't do that Dave",
                "locations": [
                  {
                    "line": 1,
                    "column": 29
                  }
                ],
                "path": [
                  "list",
                  1,
                  "item"
                ]
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
