use std::net::SocketAddr;

use grafbase_engine_integration_tests::{runtime, Engine, EngineBuilder, ResponseExt};
use serde_json::json;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

#[test]
fn simple_defer_test() {
    runtime().block_on(async {
        let mock_server = wiremock::MockServer::start().await;
        let engine = build_engine(petstore_schema(mock_server.address())).await;

        Mock::given(method("GET"))
            .and(path("/pet/123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(doggo(123, "Immediate Doggo")))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/pet/456"))
            .respond_with(ResponseTemplate::new(200).set_body_json(doggo(456, "Deferred Doggo")))
            .mount(&mock_server)
            .await;

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
                .collect()
                .await
                .into_iter()
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
            "hasNext": true,
            "path": [
              "petstore"
            ]
          }
        ]
        "###
        );
    });
}

// TODO test defer on non-streaming request
//      also test defer in places it shouldn't be used
//      todo: some general tests of spreading would also be good...
//      defer on inline fragments with typename
//      defer on named fragments

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
