//! General tests of GraphQL execution in Grafbase engine
//!
//! I wanted to call this module `graphql` but decided that could be confusing with the
//! GraphQL engine.
//!
//! A lot of these make use of OpenAPI at the moment but only because it's easy to
//! set up an OpenAPI connector.  There's no real reason they need to.
use std::net::SocketAddr;

use grafbase_engine_integration_tests::{runtime, Engine, EngineBuilder, ResponseExt};
use serde_json::json;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

#[test]
fn aliases() {
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
                "#,
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
              "veryGoodDoggo": {
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
