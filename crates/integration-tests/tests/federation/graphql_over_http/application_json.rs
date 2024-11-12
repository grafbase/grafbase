use super::APPLICATION_JSON;
use engine_v2::Engine;
use graphql_mocks::FakeGithubSchema;
use integration_tests::{federation::EngineV2Ext, runtime};

// If the URL is not used for other purposes, the server SHOULD use a 4xx status code to respond to a request that is not a well-formed GraphQL-over-HTTP request.
#[rstest::rstest]
#[case::get(http::Method::GET)]
#[case::post(http::Method::POST)]
fn ill_formed_graphql_over_http_request(#[case] method: http::Method) {
    runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(FakeGithubSchema).build().await;

        let response = engine
            .raw_execute(
                http::Request::builder()
                    .uri("http://localhost/graphql")
                    .method(method)
                    .header(http::header::CONTENT_TYPE, APPLICATION_JSON)
                    .header(http::header::ACCEPT, APPLICATION_JSON)
                    .body(Vec::from(br###"{}"###))
                    .unwrap(),
            )
            .await;
        let status = response.status();
        let body: serde_json::Value = serde_json::from_slice(&response.into_body()).unwrap();
        insta::assert_json_snapshot!(body, @r###"
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
        "###);
        assert_eq!(status, 400);
    })
}

// The server SHOULD NOT use a 4xx or 5xx status code for a response to a well-formed GraphQL-over-HTTP request.
//
// Note: For compatibility with legacy servers, this specification allows the use of 4xx or 5xx status codes for a
// failed well-formed GraphQL-over-HTTP request where the response uses the application/json media type, but it is strongly discouraged.
#[rstest::rstest]
#[case::get(http::Method::GET)]
#[case::post(http::Method::POST)]
fn request_error(#[case] method: http::Method) {
    runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(FakeGithubSchema).build().await;

        let response = engine
            .execute(method, "query { unknown }")
            .header(http::header::ACCEPT, APPLICATION_JSON)
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "errors": [
            {
              "message": "Query does not have a field named 'unknown'",
              "locations": [
                {
                  "line": 1,
                  "column": 9
                }
              ],
              "extensions": {
                "code": "OPERATION_VALIDATION_ERROR"
              }
            }
          ]
        }
        "###);
        assert_eq!(response.status, 200);
    })
}

// The server SHOULD NOT use a 4xx or 5xx status code for a response to a well-formed GraphQL-over-HTTP request.
//
// Note: For compatibility with legacy servers, this specification allows the use of 4xx or 5xx status codes for a
// failed well-formed GraphQL-over-HTTP request where the response uses the application/json media type, but it is strongly discouraged.
#[rstest::rstest]
#[case::get(http::Method::GET)]
#[case::post(http::Method::POST)]
fn field_error(#[case] method: http::Method) {
    runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(FakeGithubSchema).build().await;

        let response = engine
            .execute(method, "query { fail }")
            .header(http::header::ACCEPT, APPLICATION_JSON)
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "fail",
              "path": [
                "fail"
              ],
              "extensions": {
                "code": "SUBGRAPH_ERROR"
              }
            }
          ]
        }
        "###);
        assert_eq!(response.status, 200);
    })
}
