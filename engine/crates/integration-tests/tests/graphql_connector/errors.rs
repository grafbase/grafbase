use graphql_mocks::{ErrorSchema, MockGraphQlServer};
use integration_tests::{runtime, udfs::RustUdfs, EngineBuilder, ResponseExt};
use runtime::udf::UdfResponse;
use serde_json::json;

#[test]
fn graphql_connector_error_propagation_namespaced() {
    // Tests the case where we're joining onto a GraphQL connector, but that GraphQL connector
    // returns errors
    runtime().block_on(async {
        let graphql_mock = MockGraphQlServer::new(ErrorSchema::default()).await;
        let port = graphql_mock.port();

        let schema = format!(
            r#"
            extend schema
                @graphql(
                    name: "errors",
                    namespace: true,
                    url: "http://127.0.0.1:{port}",
                )
            "#
        );

        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(RustUdfs::new().resolver("joinContainer", UdfResponse::Success(json!({}))))
            .build()
            .await;

        insta::assert_json_snapshot!(
            engine
                .execute(r#"
                query {
                    errorsOne: errors {
                        brokenObjectList(error: "objectListError") {
                            brokenField
                        }
                    }
                    errorsTwo: errors {
                        brokenObjectList(error: "objectListError") {
                            brokenField
                        }
                    }
                }
                "#)
                .await
                .into_value(),
                @r###"
        {
          "data": {
            "errors": {
              "brokenObjectList": [
                null,
                null
              ]
            }
          },
          "errors": [
            {
              "message": "objectError",
              "path": [
                "errors",
                "brokenObjectList",
                0,
                "brokenField"
              ]
            },
            {
              "message": "objectError",
              "path": [
                "errors",
                "brokenObjectList",
                1,
                "brokenField"
              ]
            }
          ]
        }
        "###
        );
    });
}

#[test]
fn graphql_connector_error_propagation_unnamespaced() {
    // Tests the case where we're joining onto a GraphQL connector, but that GraphQL connector
    // returns errors
    runtime().block_on(async {
        let graphql_mock = MockGraphQlServer::new(ErrorSchema::default()).await;
        let port = graphql_mock.port();

        let schema = format!(
            r#"
            extend schema
                @graphql(
                    name: "errors",
                    namespace: false,
                    url: "http://127.0.0.1:{port}",
                )
            "#
        );

        let engine = EngineBuilder::new(schema)
            .with_custom_resolvers(RustUdfs::new().resolver("joinContainer", UdfResponse::Success(json!({}))))
            .build()
            .await;

        insta::assert_json_snapshot!(
            engine
                .execute(r#"
                query {
                    fieldOne: brokenObjectList(error: "objectListError") {
                        brokenField
                    }
                    fieldTwo: brokenObjectList(error: "objectListError") {
                        brokenField
                    }
                }
                "#)
                .await
                .into_value(),
                @r###"
        {
          "data": {
            "brokenObjectList": [
              null,
              null
            ]
          },
          "errors": [
            {
              "message": "objectListError",
              "path": [
                "brokenObjectList",
                0,
                "brokenField"
              ]
            },
            {
              "message": "objectListError",
              "path": [
                "brokenObjectList",
                1,
                "brokenField"
              ]
            }
          ]
        }
        "###
        );
    });
}
