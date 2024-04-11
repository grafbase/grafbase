use graphql_mocks::{ErrorSchema, FakeGithubSchema, MockGraphQlServer};
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
            "errorsOne": {
              "brokenObjectList": [
                null,
                null
              ]
            },
            "errorsTwo": {
              "brokenObjectList": [
                null,
                null
              ]
            }
          },
          "errors": [
            {
              "message": "objectListError",
              "path": [
                "errorsOne",
                "brokenObjectList",
                0,
                "brokenField"
              ]
            },
            {
              "message": "objectListError",
              "path": [
                "errorsOne",
                "brokenObjectList",
                1,
                "brokenField"
              ]
            },
            {
              "message": "objectListError",
              "path": [
                "errorsTwo",
                "brokenObjectList",
                0,
                "brokenField"
              ]
            },
            {
              "message": "objectListError",
              "path": [
                "errorsTwo",
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
fn graphql_connector_error_propagation_namespaced_but_no_grouping() {
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
                    errors {
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
              "message": "objectListError",
              "path": [
                "errors",
                "brokenObjectList",
                0,
                "brokenField"
              ]
            },
            {
              "message": "objectListError",
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
            "fieldOne": [
              null,
              null
            ],
            "fieldTwo": [
              null,
              null
            ]
          },
          "errors": [
            {
              "message": "objectListError",
              "path": [
                "fieldOne",
                0,
                "brokenField"
              ]
            },
            {
              "message": "objectListError",
              "path": [
                "fieldOne",
                1,
                "brokenField"
              ]
            },
            {
              "message": "objectListError",
              "path": [
                "fieldTwo",
                0,
                "brokenField"
              ]
            },
            {
              "message": "objectListError",
              "path": [
                "fieldTwo",
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
fn graphql_connector_error_propagation_unnamespaced_no_grouping() {
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
                    brokenObjectList(error: "objectListError") {
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

#[test]
fn should_populate_response_content_extension_on_http_errors() {
    runtime().block_on(async {
        let graphql_mock = MockGraphQlServer::new(FakeGithubSchema).await;
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

        let engine = EngineBuilder::new(schema).build().await;
        let value = engine
            .execute(
                r#"
                query {
                    one: serverVersion
                }
            "#,
            )
            .await
            .into_value();

        // Sanity check
        insta::assert_json_snapshot!(
            value,
            @r###"
        {
          "data": {
            "one": "1"
          }
        }
        "###
        );

        graphql_mock.force_next_response((http::StatusCode::INTERNAL_SERVER_ERROR, "something failed"));
        let value = engine
            .execute(
                r#"
                query {
                    one: serverVersion
                }
            "#,
            )
            .await
            .into_value();

        // Sanity check
        insta::assert_json_snapshot!(
            value,
            @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "received an unexpected status from the downstream server: 500",
              "locations": [
                {
                  "line": 3,
                  "column": 21
                }
              ],
              "path": [
                "one"
              ],
              "extensions": {
                "response_content": "something failed"
              }
            }
          ]
        }
        "###
        );
    });
}
