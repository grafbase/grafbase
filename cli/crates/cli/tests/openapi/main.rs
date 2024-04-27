#![allow(unused_crate_dependencies)]
#[path = "../utils/mod.rs"]
mod utils;

mod headers;
mod http_spy;
mod introspection_headers;
mod remote_unions;
mod transforms;

use std::{fmt::Display, net::SocketAddr};

use backend::project::GraphType;
use serde_json::{json, Value};
use utils::{async_client::AsyncClient, environment::Environment};
use wiremock::{
    matchers::{header, method, path},
    Mock, ResponseTemplate,
};

use self::http_spy::ReceivedBodiesExt;

#[tokio::test(flavor = "multi_thread")]
async fn openapi_test() {
    let mock_server = wiremock::MockServer::start().await;
    mount_petstore_spec(&mock_server).await;

    let mut env = Environment::init_async().await;
    let client = start_grafbase(&mut env, petstore_schema(mock_server.address())).await;

    Mock::given(method("GET"))
        .and(path("/pet/123"))
        .and(header("authorization", "Bearer BLAH"))
        .respond_with(ResponseTemplate::new(200).set_body_json(doggie()))
        .mount(&mock_server)
        .await;

    insta::assert_yaml_snapshot!(
        client
            .gql::<Value>(
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
            .await,
        @r###"
    ---
    data:
      petstore:
        pet:
          id: 123
          name: doggie
          status: AVAILABLE
    "###
    );

    let mock_guard = Mock::given(method("PUT"))
        .and(path("/pet"))
        .and(header("authorization", "Bearer BLAH"))
        .respond_with(ResponseTemplate::new(200).set_body_json(doggie()))
        .mount_as_scoped(&mock_server)
        .await;

    insta::assert_yaml_snapshot!(
        client
            .gql::<Value>(
                r#"
                    mutation {
                        petstore {
                            updatePet(input: {
                                id: 123
                                name: "Doggie"
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
            .await,
        @r###"
    ---
    data:
      petstore:
        updatePet:
          id: 123
          name: doggie
          status: AVAILABLE
    "###
    );

    insta::assert_yaml_snapshot!(mock_guard.received_json_bodies().await, @r###"
    ---
    - status: available
      tags: []
      photoUrls: []
      category: {}
      name: Doggie
      id: 123
    "###);
}

#[tokio::test(flavor = "multi_thread")]
async fn openapi_flat_namespace() {
    let mock_server = wiremock::MockServer::start().await;
    mount_petstore_spec(&mock_server).await;

    let mut env = Environment::init_async().await;
    let client = start_grafbase(&mut env, no_namespace_schema(mock_server.address())).await;

    Mock::given(method("GET"))
        .and(path("/pet/123"))
        .and(header("authorization", "Bearer BLAH"))
        .respond_with(ResponseTemplate::new(200).set_body_json(doggie()))
        .mount(&mock_server)
        .await;

    let value = client
        .gql::<Value>(
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
        .await;

    insta::with_settings!({sort_maps => true}, {
        insta::assert_yaml_snapshot!(
            value,
            @r###"
        ---
        data:
          pet:
            id: 123
            name: doggie
            status: AVAILABLE
        "###
        );
    });

    let mock_guard = Mock::given(method("PUT"))
        .and(path("/pet"))
        .and(header("authorization", "Bearer BLAH"))
        .respond_with(ResponseTemplate::new(200).set_body_json(doggie()))
        .mount_as_scoped(&mock_server)
        .await;

    let value = client
        .gql::<Value>(
            r#"
            mutation {
                updatePet(input: {
                    id: 123
                    name: "Doggie"
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
        .await;

    insta::with_settings!({sort_maps => true}, {
        insta::assert_yaml_snapshot!(
            value,
            @r###"
    ---
    data:
      updatePet:
        id: 123
        name: doggie
        status: AVAILABLE
    "###
        );
    });

    let value = mock_guard.received_json_bodies().await;
    insta::with_settings!({sort_maps => true}, {
    insta::assert_yaml_snapshot!(value, @r###"
    ---
    - category: {}
      id: 123
      name: Doggie
      photoUrls: []
      status: available
      tags: []
    "###);
    });
}

async fn start_grafbase(env: &mut Environment, schema: impl AsRef<str> + Display) -> AsyncClient {
    env.grafbase_init(GraphType::Standalone);
    env.write_schema(schema);
    env.set_variables([("API_KEY", "BLAH")]);
    env.grafbase_dev_watch();

    let client = env.create_async_client().with_api_key();

    client.poll_endpoint(30, 300).await;

    client
}

fn petstore_schema(address: &SocketAddr) -> String {
    format!(
        r#"
          extend schema
          @openapi(
            name: "petstore",
            namespace: true,
            url: "http://{address}",
            schema: "http://{address}/spec.json",
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
            schema: "http://{address}/spec.json",
            headers: [{{ name: "authorization", value: "Bearer {{{{ env.API_KEY }}}}" }}],
          )
        "#
    )
}

async fn mount_petstore_spec(server: &wiremock::MockServer) {
    Mock::given(method("GET"))
        .and(path("spec.json"))
        .respond_with(ResponseTemplate::new(200).set_body_string(include_str!("petstore.json")))
        .mount(server)
        .await;
}

fn doggie() -> serde_json::Value {
    json!({
        "id": 123,
        "name": "doggie",
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
