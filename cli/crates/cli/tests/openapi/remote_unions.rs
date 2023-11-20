use std::net::SocketAddr;

use crate::utils::environment::Environment;
use serde_json::{json, Value};
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use super::start_grafbase;

#[tokio::test(flavor = "multi_thread")]
async fn remote_unions_test() {
    let mock_server = wiremock::MockServer::start().await;
    mount_remote_union_spec(&mock_server).await;

    let mut env = Environment::init_async().await;
    let client = start_grafbase(&mut env, schema(mock_server.address())).await;

    Mock::given(method("GET"))
        .and(path("/pets"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!([doggie(json!("Mrs Krabappel")), doggie(json!({"id": 123}))])),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    insta::assert_yaml_snapshot!(
        client
            .gql::<Value>(
                r"
                    query {
                        petstore {
                            pets {
                                id
                                owner {
                                    __typename
                                    ... on PetstorePerson {
                                        id
                                    }
                                    ... on PetstoreString {
                                        data
                                    }
                                }
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
        pets:
          - id: 123
            owner:
              __typename: PetstoreString
              data: Mrs Krabappel
          - id: 123
            owner:
              __typename: PetstorePerson
              id: 123
    "###
    );

    Mock::given(method("GET"))
        .and(path("/mainOwner"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!("Mrs Krabappel")))
        .expect(1)
        .mount(&mock_server)
        .await;

    insta::assert_yaml_snapshot!(
        client
            .gql::<Value>(
                r"
                    query {
                        petstore {
                            owner {
                                __typename
                                ... on PetstorePerson {
                                    id
                                }
                                ... on PetstoreString {
                                    data
                                }
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
        owner:
          __typename: PetstoreString
          data: Mrs Krabappel
    "###
    );
}

fn schema(address: &SocketAddr) -> String {
    format!(
        r#"
          extend schema
          @openapi(
            name: "petstore",
            namespace: true,
            url: "http://{address}",
            schema: "http://{address}/spec.json",
          )
        "#
    )
}

async fn mount_remote_union_spec(server: &wiremock::MockServer) {
    Mock::given(method("GET"))
        .and(path("spec.json"))
        .respond_with(ResponseTemplate::new(200).set_body_string(include_str!("remote_union_spec.json")))
        .mount(server)
        .await;
}

#[allow(clippy::needless_pass_by_value)]
fn doggie(owner: serde_json::Value) -> serde_json::Value {
    json!({ "id": 123, "owner": owner })
}
