use std::net::SocketAddr;

use integration_tests::{runtime, EngineBuilder, ResponseExt};
use serde_json::json;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

#[test]
fn remote_unions_test() {
    runtime().block_on(async {
        let mock_server = wiremock::MockServer::start().await;
        let engine = EngineBuilder::new(schema(mock_server.address()))
            .with_openapi_schema(
                "http://example.com/remote_unions.json",
                include_str!("remote_union_spec.json"),
            )
            .build()
            .await;

        Mock::given(method("GET"))
            .and(path("/pets"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!([doggo(json!("Mrs Krabappel")), doggo(json!({"id": 123}))])),
            )
            .expect(1)
            .mount(&mock_server)
            .await;

        insta::assert_json_snapshot!(
            engine
                .execute(
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
                .await
                .into_value(),
            @r###"
        {
          "data": {
            "petstore": {
              "pets": [
                {
                  "id": 123,
                  "owner": {
                    "__typename": "PetstoreString",
                    "data": "Mrs Krabappel"
                  }
                },
                {
                  "id": 123,
                  "owner": {
                    "__typename": "PetstorePerson",
                    "id": 123
                  }
                }
              ]
            }
          }
        }
        "###
        );

        Mock::given(method("GET"))
            .and(path("/mainOwner"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!("Mrs Krabappel")))
            .expect(1)
            .mount(&mock_server)
            .await;

        insta::assert_json_snapshot!(
            engine
                .execute(
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
                .await
                .into_value(),
            @r###"
        {
          "data": {
            "petstore": {
              "owner": {
                "__typename": "PetstoreString",
                "data": "Mrs Krabappel"
              }
            }
          }
        }
        "###
        );
    });
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

#[allow(clippy::needless_pass_by_value)]
fn doggo(owner: serde_json::Value) -> serde_json::Value {
    json!({ "id": 123, "owner": owner })
}
