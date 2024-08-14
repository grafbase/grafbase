use engine_v2::Engine;
use graphql_mocks::FakeGithubSchema;
use integration_tests::{federation::EngineV2Ext, runtime};

#[test]
fn success() {
    runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(FakeGithubSchema).build().await;

        // Query should work
        let response = engine
            .raw_execute(
                http::Request::builder()
                    .method(http::Method::POST)
                    .header(http::header::ACCEPT, "application/json")
                    .header(http::header::CONTENT_TYPE, "application/json")
                    .body(
                        serde_json::to_vec(&serde_json::json!([
                            {"query": "{ first: __typename }"},
                            {"query": "{ second: __typename }"},
                        ]))
                        .unwrap(),
                    )
                    .unwrap(),
            )
            .await;

        let status = response.status();
        let body: serde_json::Value = serde_json::from_slice(&response.into_body().into_bytes().unwrap()).unwrap();
        insta::assert_json_snapshot!(body, @r###"
        [
          {
            "data": {
              "first": "Query"
            }
          },
          {
            "data": {
              "second": "Query"
            }
          }
        ]
        "###);
        assert_eq!(status, 200);
    })
}

#[test]
fn invalid_request() {
    runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(FakeGithubSchema).build().await;

        // Query should work
        let response = engine
            .raw_execute(
                http::Request::builder()
                    .method(http::Method::POST)
                    .header(http::header::ACCEPT, "application/json")
                    .header(http::header::CONTENT_TYPE, "application/json")
                    .body(
                        serde_json::to_vec(&serde_json::json!([
                            {"query": "{ first: __typename }"},
                            {"qu": "???"},
                        ]))
                        .unwrap(),
                    )
                    .unwrap(),
            )
            .await;

        let status = response.status();
        let body: serde_json::Value = serde_json::from_slice(&response.into_body().into_bytes().unwrap()).unwrap();
        insta::assert_json_snapshot!(body, @r###"
        [
          {
            "data": {
              "first": "Query"
            }
          },
          {
            "errors": [
              {
                "message": "Missing query",
                "extensions": {
                  "code": "BAD_REQUEST"
                }
              }
            ]
          }
        ]
        "###);
        assert_eq!(status, 400);
    })
}

#[test]
fn request_error() {
    runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(FakeGithubSchema).build().await;

        // Query should work
        let response = engine
            .raw_execute(
                http::Request::builder()
                    .method(http::Method::POST)
                    .header(http::header::ACCEPT, "application/json")
                    .header(http::header::CONTENT_TYPE, "application/json")
                    .body(
                        serde_json::to_vec(&serde_json::json!([
                            {"query": "{ first: __typename }"},
                            {"query": "{ unknown }"},
                        ]))
                        .unwrap(),
                    )
                    .unwrap(),
            )
            .await;

        let status = response.status();
        let body: serde_json::Value = serde_json::from_slice(&response.into_body().into_bytes().unwrap()).unwrap();
        insta::assert_json_snapshot!(body, @r###"
        [
          {
            "data": {
              "first": "Query"
            }
          },
          {
            "errors": [
              {
                "message": "Query does not have a field named 'unknown'",
                "locations": [
                  {
                    "line": 1,
                    "column": 3
                  }
                ],
                "extensions": {
                  "code": "OPERATION_VALIDATION_ERROR"
                }
              }
            ]
          }
        ]
        "###);
        assert_eq!(status, 200);
    })
}
