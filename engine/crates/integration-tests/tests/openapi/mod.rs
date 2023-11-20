mod headers;
mod http_spy;
mod remote_unions;
mod transforms;

use std::net::SocketAddr;

use integration_tests::{runtime, Engine, EngineBuilder, ResponseExt};
use serde_json::json;
use wiremock::{
    matchers::{header, method, path},
    Mock, ResponseTemplate,
};

use self::http_spy::ReceivedBodiesExt;

#[test]
fn openapi_test() {
    runtime().block_on(async {
        let mock_server = wiremock::MockServer::start().await;
        let engine = build_engine(petstore_schema(mock_server.address())).await;

        Mock::given(method("GET"))
            .and(path("/pet/123"))
            .and(header("authorization", "Bearer BLAH"))
            .respond_with(ResponseTemplate::new(200).set_body_json(doggo()))
            .mount(&mock_server)
            .await;

        insta::assert_json_snapshot!(
            engine
                .execute(
                r"
                    query {
                        petstore {
                            pet(petId: 123) {
                                id
                                name
                                status
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
                "name": "doggo",
                "status": "AVAILABLE"
              }
            }
          }
        }
        "###
        );

        let mock_guard = Mock::given(method("PUT"))
            .and(path("/pet"))
            .and(header("authorization", "Bearer BLAH"))
            .respond_with(ResponseTemplate::new(200).set_body_json(doggo()))
            .mount_as_scoped(&mock_server)
            .await;

        insta::assert_json_snapshot!(
            engine
                .execute(
                r#"
                    mutation {
                        petstore {
                            updatePet(input: {
                                id: 123
                                name: "doggo"
                                status: AVAILABLE
                                tags: []
                                photoUrls: []
                                category: {}
                            }) {
                                id
                                name
                                status
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
              "updatePet": {
                "id": 123,
                "name": "doggo",
                "status": "AVAILABLE"
              }
            }
          }
        }
        "###
            );

        insta::assert_json_snapshot!(mock_guard.received_json_bodies().await, @r###"
        [
          {
            "category": {},
            "id": 123,
            "name": "doggo",
            "photoUrls": [],
            "status": "available",
            "tags": []
          }
        ]
        "###);
    });
}

#[test]
fn openapi_flat_namespace() {
    runtime().block_on(async {
        let mock_server = wiremock::MockServer::start().await;
        let engine = build_engine(no_namespace_schema(mock_server.address())).await;

        Mock::given(method("GET"))
            .and(path("/pet/123"))
            .and(header("authorization", "Bearer BLAH"))
            .respond_with(ResponseTemplate::new(200).set_body_json(doggo()))
            .mount(&mock_server)
            .await;

        insta::assert_json_snapshot!(
            engine
                .execute(
                    r"
                    query {
                        pet(petId: 123) {
                            id
                            name
                            status
                        }
                    }
                ",
                )
                .await
                .into_value(),
            @r###"
        {
          "data": {
            "pet": {
              "id": 123,
              "name": "doggo",
              "status": "AVAILABLE"
            }
          }
        }
        "###
        );

        let mock_guard = Mock::given(method("PUT"))
            .and(path("/pet"))
            .and(header("authorization", "Bearer BLAH"))
            .respond_with(ResponseTemplate::new(200).set_body_json(doggo()))
            .mount_as_scoped(&mock_server)
            .await;

        insta::assert_json_snapshot!(
            engine
                .execute(
                    r#"
                    mutation {
                        updatePet(input: {
                            id: 123
                            name: "doggo"
                            status: AVAILABLE
                            tags: []
                            photoUrls: []
                            category: {}
                        }) {
                            id
                            name
                            status
                        }
                    }
                "#,
                )
                .await
                .into_value(),
            @r###"
        {
          "data": {
            "updatePet": {
              "id": 123,
              "name": "doggo",
              "status": "AVAILABLE"
            }
          }
        }
        "###
        );

        insta::assert_json_snapshot!(mock_guard.received_json_bodies().await, @r###"
        [
          {
            "category": {},
            "id": 123,
            "name": "doggo",
            "photoUrls": [],
            "status": "available",
            "tags": []
          }
        ]
        "###);
    });
}

async fn build_engine(schema: String) -> Engine {
    EngineBuilder::new(schema)
        .with_openapi_schema("http://example.com/petstore.json", include_str!("petstore.json"))
        .with_env_var("API_KEY", "BLAH")
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
            headers: [{{ name: "authorization", value: "Bearer {{{{ env.API_KEY }}}}" }}],
          )
        "#
    )
}

fn no_namespace_schema(address: &SocketAddr) -> String {
    format!(
        r#"
          extend schema
          @openapi(
            name: "petstore",
            namespace: false,
            url: "http://{address}",
            schema: "http://example.com/petstore.json",
            headers: [{{ name: "authorization", value: "Bearer {{{{ env.API_KEY }}}}" }}],
          )
        "#
    )
}

fn doggo() -> serde_json::Value {
    json!({
        "id": 123,
        "name": "doggo",
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
