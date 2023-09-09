//! Tests of transforms on openapi

use std::net::SocketAddr;

use cynic::QueryBuilder;
use cynic_introspection::IntrospectionQuery;
use grafbase_engine_integration_tests::{runtime, EngineBuilder, ResponseExt};

#[test]
fn test_openapi_with_transforms() {
    runtime().block_on(async {
        let mock_server = wiremock::MockServer::start().await;
        let engine = EngineBuilder::new(petstore_schema_with_transforms(mock_server.address()))
            .with_openapi_schema("http://example.com/petstore.json", include_str!("transforms_spec.json"))
            .with_env_var("API_KEY", "BLAH")
            .build()
            .await;

        let introspection_query = IntrospectionQuery::build(());
        let response = engine
            .execute(introspection_query)
            .await
            .into_data::<IntrospectionQuery>();

        insta::assert_snapshot!(response.into_schema().unwrap().to_sdl(), @r###"
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
    });
}

fn petstore_schema_with_transforms(address: &SocketAddr) -> String {
    format!(
        r#"
          extend schema
          @openapi(
            name: "petstore",
            url: "http://{address}",
            schema: "http://example.com/petstore.json",
            transforms: {{
              exclude: [
                "Pet.owner"
              ]
            }}
          )
        "#
    )
}
