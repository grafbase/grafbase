//! Various tests of errors during execution

use std::net::SocketAddr;

use integration_tests::{runtime, udfs::RustUdfs, Engine, EngineBuilder, ResponseExt};
use runtime::udf::{CustomResolverRequestPayload, CustomResolverResponse};
use serde_json::json;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

#[test]
fn error_propagation_openapi() {
    runtime().block_on(async {
        let mock_server = wiremock::MockServer::start().await;
        let engine = build_petstore_engine(petstore_schema(mock_server.address())).await;

        // We only set up one the pets we request, so we should get
        // one pet back and a null on the other (with an error explaining why)
        mock_doggo(&mock_server, 123, "Immediate Doggo").await;

        insta::assert_json_snapshot!(
            engine
                .execute(
                r"
                    query {
                        petstore {
                            goodDoggo: pet(petId: 123) {
                                id
                                name
                            }
                            veryGoodDoggo: pet(petId: 456) {
                                id
                                name
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
              "goodDoggo": {
                "id": 123,
                "name": "Immediate Doggo"
              },
              "veryGoodDoggo": null
            }
          },
          "errors": [
            {
              "message": "Received an unexpected status from the downstream server: 404 Not Found",
              "locations": [
                {
                  "line": 8,
                  "column": 29
                }
              ],
              "path": [
                "petstore",
                "veryGoodDoggo"
              ]
            }
          ]
        }
        "###
        );
    });
}

#[test]
fn querying_unknown_field() {
    runtime().block_on(async {
        let mock_server = wiremock::MockServer::start().await;
        let engine = build_petstore_engine(petstore_schema(mock_server.address())).await;

        insta::assert_json_snapshot!(
            engine
                .execute(
                r"
                    query {
                        petstore {
                          someNonsenseField {
                            id
                          }
                        }
                    }
                ",
                )
                .await
                .into_value(),
            @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Unknown field \"someNonsenseField\" on type \"PetstoreQuery\".",
              "locations": [
                {
                  "line": 4,
                  "column": 27
                }
              ]
            }
          ]
        }
        "###
        );
    });
}

#[test]
fn error_handling_scalar_custom_resolver() {
    runtime().block_on(async {
        let schema = r#"
            type Query {
                error: String @resolver(name: "error")
            }
        "#;
        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(RustUdfs::new().resolver(
                "error",
                CustomResolverResponse::GraphQLError {
                    message: "Shits on fire yo".into(),
                    extensions: None,
                },
            ))
            .build()
            .await;

        insta::assert_json_snapshot!(
            engine.execute("query { error }",).await.into_value(),
            @r###"
        {
          "data": {
            "error": null
          },
          "errors": [
            {
              "message": "Shits on fire yo",
              "locations": [
                {
                  "line": 1,
                  "column": 9
                }
              ],
              "path": [
                "error"
              ]
            }
          ]
        }
        "###
        );
    });
}

#[test]
fn error_handling_list_custom_resolver() {
    // Tests handling of errors on list fields
    runtime().block_on(async {
        let schema = r#"
            type Query {
                error: [String] @resolver(name: "error")
            }
        "#;
        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(RustUdfs::new().resolver(
                "error",
                CustomResolverResponse::GraphQLError {
                    message: "Shits on fire yo".into(),
                    extensions: None,
                },
            ))
            .build()
            .await;

        insta::assert_json_snapshot!(
            engine.execute("query { error }",).await.into_value(),
            @r###"
        {
          "data": {
            "error": null
          },
          "errors": [
            {
              "message": "Shits on fire yo",
              "locations": [
                {
                  "line": 1,
                  "column": 9
                }
              ],
              "path": [
                "error"
              ]
            }
          ]
        }
        "###
        );
    });
}

#[test]
fn error_handling_list_propagation() {
    // Tests that errors inside list items propagate to the list item and not the list
    runtime().block_on(async {
        let schema = r#"
          type Query {
              list: [ObjectWithErrors]! @resolver(name: "list")
          }

          type ObjectWithErrors {
              hello: String! @resolver(name: "item")
          }
        "#;
        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(
                RustUdfs::new()
                    .resolver("list", CustomResolverResponse::Success(json!([{"id": 1}, {"id": 2}])))
                    .resolver("item", |payload: CustomResolverRequestPayload| {
                        if payload.parent.unwrap()["id"] == json!(1) {
                            Ok(CustomResolverResponse::Success(json!("world")))
                        } else {
                            Ok(CustomResolverResponse::GraphQLError {
                                message: "get out of my pub".into(),
                                extensions: None,
                            })
                        }
                    }),
            )
            .build()
            .await;

        insta::assert_json_snapshot!(
            engine.execute("query { list { hello }}",).await.into_value(),
            @r###"
        {
          "data": {
            "list": [
              {
                "hello": "world"
              },
              null
            ]
          },
          "errors": [
            {
              "message": "get out of my pub",
              "locations": [
                {
                  "line": 1,
                  "column": 16
                }
              ],
              "path": [
                "list",
                1,
                "hello"
              ]
            }
          ]
        }
        "###
        );
    });
}

async fn build_petstore_engine(schema: String) -> Engine {
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
