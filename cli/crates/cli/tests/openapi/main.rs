#![allow(unused_crate_dependencies)]
#[path = "../utils/mod.rs"]
mod utils;

mod introspection_headers;
mod remote_unions;

use std::net::SocketAddr;

use backend::project::ConfigType;
use crossbeam_channel::{Receiver, Sender};
use serde_json::{json, Value};
use utils::{async_client::AsyncClient, environment::Environment};
use wiremock::{
    matchers::{header, method, path},
    Match, Mock, ResponseTemplate,
};

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
                r#"
                    query {
                        petstore {
                            pet(petId: 123) {
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
        pet:
          id: 123
          name: doggie
          status: AVAILABLE
    "###
    );

    let request_body_spy = RequestBodySpy::new();

    Mock::given(method("PUT"))
        .and(path("/pet"))
        .and(header("authorization", "Bearer BLAH"))
        .and(request_body_spy.clone())
        .respond_with(ResponseTemplate::new(200).set_body_json(doggie()))
        .mount(&mock_server)
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

    insta::assert_yaml_snapshot!(request_body_spy.drain_requests(), @r###"
    ---
    - category: {}
      id: 123
      name: Doggie
      photoUrls: []
      status: available
      tags: []
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

    insta::assert_yaml_snapshot!(
        client
            .gql::<Value>(
                r#"
                    query {
                        pet(petId: 123) {
                            id
                            name
                            status
                        }
                    }
                "#,
            )
            .await,
        @r###"
    ---
    data:
      pet:
        id: 123
        name: doggie
        status: AVAILABLE
    "###
    );

    let request_body_spy = RequestBodySpy::new();

    Mock::given(method("PUT"))
        .and(path("/pet"))
        .and(header("authorization", "Bearer BLAH"))
        .and(request_body_spy.clone())
        .respond_with(ResponseTemplate::new(200).set_body_json(doggie()))
        .mount(&mock_server)
        .await;

    insta::assert_yaml_snapshot!(
        client
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
            .await,
        @r###"
    ---
    data:
      updatePet:
        id: 123
        name: doggie
        status: AVAILABLE
    "###
    );

    insta::assert_yaml_snapshot!(request_body_spy.drain_requests(), @r###"
    ---
    - category: {}
      id: 123
      name: Doggie
      photoUrls: []
      status: available
      tags: []
    "###);
}

#[derive(Clone)]
struct RequestBodySpy {
    receiver: Receiver<Value>,
    sender: Sender<Value>,
}

impl RequestBodySpy {
    pub fn new() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        RequestBodySpy { receiver, sender }
    }

    pub fn drain_requests(&self) -> Vec<Value> {
        self.receiver.try_iter().collect()
    }
}

impl Match for RequestBodySpy {
    fn matches(&self, request: &wiremock::Request) -> bool {
        self.sender
            .send(request.body_json().expect("A JSON Body"))
            .expect("channel to be open");

        true
    }
}

async fn start_grafbase(env: &mut Environment, schema: impl AsRef<str>) -> AsyncClient {
    env.grafbase_init(ConfigType::GraphQL);
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
            namespace: "petstore",
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
