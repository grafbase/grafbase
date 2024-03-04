//! Tests of transforms on openapi

use std::net::SocketAddr;

use cynic::QueryBuilder;
use cynic_introspection::IntrospectionQuery;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::utils::environment::Environment;

use super::start_grafbase;

#[ctor::ctor]
fn setup_rustls() {
    rustls::crypto::ring::default_provider().install_default().unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_openapi_with_transforms() {
    let mock_server = wiremock::MockServer::start().await;
    mount_spec(&mock_server).await;

    let mut env = Environment::init_async().await;
    let client = start_grafbase(&mut env, petstore_schema_with_transforms(mock_server.address())).await;

    let introspection_query = IntrospectionQuery::build(());
    let response = client
        .gql::<cynic::GraphQlResponse<IntrospectionQuery>>(introspection_query.query)
        .await;

    insta::assert_snapshot!(response.data.unwrap().into_schema().unwrap().to_sdl(), @r###"
    type PetstorePet {
      id: Int!
      name: String
    }

    type PetstoreQuery {
      pets: [PetstorePet!]
    }

    type Query {
      petstore: PetstoreQuery!
    }

    "###);
}

async fn mount_spec(server: &wiremock::MockServer) {
    Mock::given(method("GET"))
        .and(path("spec.json"))
        .respond_with(ResponseTemplate::new(200).set_body_string(include_str!("transforms_spec.json")))
        .mount(server)
        .await;
}

fn petstore_schema_with_transforms(address: &SocketAddr) -> String {
    format!(
        r#"
          extend schema
          @openapi(
            name: "petstore",
            namespace: true,
            url: "http://{address}",
            schema: "http://{address}/spec.json",
            transforms: {{
              exclude: [
                "Pet.owner"
              ]
            }}
          )
        "#
    )
}
