//! Tests of passing through headers

use std::{collections::BTreeMap, net::SocketAddr};

use serde_json::Value;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::utils::environment::Environment;

use super::{doggie, mount_petstore_spec, start_grafbase};

#[tokio::test(flavor = "multi_thread")]
async fn test_header_passthrough() {
    let mock_server = wiremock::MockServer::start().await;
    mount_petstore_spec(&mock_server).await;

    let mut env = Environment::init_async().await;
    let client = start_grafbase(&mut env, petstore_schema_with_header_forwarding(mock_server.address())).await;

    let mock_guard = Mock::given(method("GET"))
        .and(path("/pet/123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(doggie()))
        .mount_as_scoped(&mock_server)
        .await;

    insta::assert_yaml_snapshot!(
        client
            .gql::<Value>(
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
            .await,
        @r###"
    ---
    data:
      petstore:
        pet:
          id: 123
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

    insta::assert_yaml_snapshot!(headers, @r###"
    ---
    - accept: "[\"*/*\"]"
      another-one: "[\"yes\"]"
      authorization: "[\"Bearer BLAH\"]"
      wow-what-a-header: "[\"isn't it the best\"]"
    "###);
}

fn petstore_schema_with_header_forwarding(address: &SocketAddr) -> String {
    format!(
        r#"
          extend schema
          @openapi(
            name: "petstore",
            namespace: true,
            url: "http://{address}",
            schema: "http://{address}/spec.json",
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
