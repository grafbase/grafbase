//! Tests of passing through headers

use std::{collections::BTreeMap, net::SocketAddr};

use integration_tests::{runtime, ResponseExt};
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use super::{build_engine, doggo};

#[test]
fn test_header_passthrough() {
    runtime().block_on(async {
        let mock_server = wiremock::MockServer::start().await;
        let engine = build_engine(petstore_schema_with_header_forwarding(mock_server.address())).await;

        let mock_guard = Mock::given(method("GET"))
            .and(path("/pet/123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(doggo()))
            .mount_as_scoped(&mock_server)
            .await;

        insta::assert_json_snapshot!(
            engine
                .execute(
                    r#"
                    query {
                        petstore {
                            pet(petId: 123) {
                                id
                            }
                        }
                    }
                "#,
                )
                .header("wow-what-a-header", "isn't it the best")
                .header("and-another-one", "yes")
                .header("a-header-that-shouldnt-be-forwarded", "ok")
                .header("Authorization", "Basic XYZ")
                .await.into_value(),
            @r###"
        {
          "data": {
            "petstore": {
              "pet": {
                "id": 123
              }
            }
          }
        }
        "###
        );

        let headers = mock_guard
            .received_requests()
            .await
            .into_iter()
            .map(|request| {
                // Wow, this is annoying...
                request
                    .headers
                    .into_iter()
                    .map(|(name, value)| (name.to_string(), value.to_string()))
                    .filter(|(name, _)| {
                        // Host changes on every test, we need to filter it out.
                        // The others are just noise so I'm also ditching them.
                        name != "host" && name != "connection" && name != "accept-encoding" && name != "mf-loop"
                    })
                    .collect::<BTreeMap<_, _>>()
            })
            .collect::<Vec<_>>();

        insta::assert_json_snapshot!(headers, @r###"
        [
          {
            "accept": "[\"*/*\"]",
            "another-one": "[\"yes\"]",
            "authorization": "[\"Bearer BLAH\"]",
            "wow-what-a-header": "[\"isn't it the best\"]"
          }
        ]
        "###);
    });
}

fn petstore_schema_with_header_forwarding(address: &SocketAddr) -> String {
    format!(
        r#"
          extend schema
          @openapi(
            name: "petstore",
            url: "http://{address}",
            schema: "http://example.com/petstore.json",
            headers: [
                {{ name: "authorization", value: "Bearer {{{{ env.API_KEY }}}}" }}
                {{ name: "Wow-what-a-header", forward: "Wow-what-a-header" }}
                {{ name: "another-one", forward: "and-another-one" }}
                {{ name: "secret-third-header", forward: "secret-third-header" }}
            ],
          )
        "#
    )
}
