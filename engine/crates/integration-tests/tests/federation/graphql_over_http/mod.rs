mod application_graphql_response_json;
mod application_json;
mod batch;

use engine_v2::Engine;
use graphql_mocks::{FakeGithubSchema, Stateful};
use integration_tests::{federation::EngineV2Ext, openid::JWKS_URI, runtime};

const APPLICATION_JSON: &str = "application/json";
const APPLICATION_GRAPHQL_RESPONSE_JSON: &str = "application/graphql-response+json";

// A server MAY forbid individual requests by a client to any endpoint for any reason, for example to require authentication or payment;
// when doing so it SHOULD use the relevant 4xx or 5xx status code. This decision SHOULD NOT be based on the contents of a well-formed GraphQL-over-HTTP request.
#[rstest::rstest]
#[case::post_json(http::Method::POST, APPLICATION_JSON)]
#[case::post_gql_json(http::Method::POST, APPLICATION_GRAPHQL_RESPONSE_JSON)]
#[case::get_json(http::Method::GET, APPLICATION_JSON)]
#[case::get_gql_json(http::Method::GET, APPLICATION_GRAPHQL_RESPONSE_JSON)]
fn authentication_returns_401(#[case] method: http::Method, #[case] accept: &'static str) {
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FakeGithubSchema)
            .with_sdl_config(format!(
                r#"extend schema @authz(providers: [
                    {{ name: "my-jwt", type: jwt, jwks: {{ url: "{JWKS_URI}" }} }}
                ])"#
            ))
            .build()
            .await;

        // Invalid request should not matter.
        let response = engine.execute(method, "").header(http::header::ACCEPT, accept).await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "errors": [
            {
              "message": "Unauthenticated",
              "extensions": {
                "code": "UNAUTHENTICATED"
              }
            }
          ]
        }
        "###);
        assert_eq!(response.status, 401);
    })
}

// Note: If a client does not supply the Accept header then the server may respond with an error, or with any content type it chooses.
// To ensure your client gets something useful, it should indicate the media types it supports.
#[test]
fn missing_accept_header() {
    runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(FakeGithubSchema).build().await;

        let response = engine
            .raw_execute(
                http::Request::builder()
                    .uri("http://localhost/graphql")
                    .method(http::Method::POST)
                    .header(http::header::CONTENT_TYPE, APPLICATION_JSON)
                    .body(br###"{"query":"{ __typename }"}"###.to_vec())
                    .unwrap(),
            )
            .await;
        let status = response.status();
        assert_eq!(
            response
                .headers()
                .get(http::header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
            Some(APPLICATION_JSON)
        );
        let body: serde_json::Value = serde_json::from_slice(&response.into_body()).unwrap();
        insta::assert_json_snapshot!(body, @r###"
        {
          "data": {
            "__typename": "Query"
          }
        }
        "###);
        assert_eq!(status, 200);
    })
}

#[test]
fn star_accept_header_should_be_accepted() {
    runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(FakeGithubSchema).build().await;

        let response = engine
            .raw_execute(
                http::Request::builder()
                    .uri("http://localhost/graphql")
                    .method(http::Method::POST)
                    .header(http::header::CONTENT_TYPE, APPLICATION_JSON)
                    .header(http::header::ACCEPT, "application/*")
                    .body(br###"{"query":"{ __typename }"}"###.to_vec())
                    .unwrap(),
            )
            .await;
        let status = response.status();
        assert_eq!(
            response
                .headers()
                .get(http::header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
            Some(APPLICATION_JSON)
        );
        let body: serde_json::Value = serde_json::from_slice(&response.into_body()).unwrap();
        insta::assert_json_snapshot!(body, @r###"
        {
          "data": {
            "__typename": "Query"
          }
        }
        "###);
        assert_eq!(status, 200);

        let response = engine
            .raw_execute(
                http::Request::builder()
                    .uri("http://localhost/graphql")
                    .method(http::Method::POST)
                    .header(http::header::CONTENT_TYPE, APPLICATION_JSON)
                    .header(http::header::ACCEPT, "*/*")
                    .body(br###"{"query":"{ __typename }"}"###.to_vec())
                    .unwrap(),
            )
            .await;
        let status = response.status();
        assert_eq!(
            response
                .headers()
                .get(http::header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
            Some(APPLICATION_JSON)
        );
        let body: serde_json::Value = serde_json::from_slice(&response.into_body()).unwrap();
        insta::assert_json_snapshot!(body, @r###"
        {
          "data": {
            "__typename": "Query"
          }
        }
        "###);
        assert_eq!(status, 200);
    })
}

// In alignment with the HTTP 1.1 Accept specification, when a client does not include at least one supported media type
// in the Accept HTTP header, the server MUST either:
//    1. Respond with a 406 Not Acceptable status code and stop processing the request (RECOMMENDED); OR
//    2. Disregard the Accept header and respond with the server's choice of media type (NOT RECOMMENDED).
#[test]
fn unsupported_accept_header() {
    runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(FakeGithubSchema).build().await;

        let response = engine
            .raw_execute(
                http::Request::builder()
                    .uri("http://localhost/graphql")
                    .method(http::Method::POST)
                    .header(http::header::CONTENT_TYPE, APPLICATION_JSON)
                    .header(http::header::ACCEPT, "application/jpeg")
                    .body(br###"{"query":"{ __typename }"}"###.to_vec())
                    .unwrap(),
            )
            .await;
        let status = response.status();
        let body: serde_json::Value = serde_json::from_slice(&response.into_body()).unwrap();
        insta::assert_json_snapshot!(body, @r###"
        {
          "errors": [
            {
              "message": "Missing or invalid Accept header. You must specify one of: '*/*', 'application/*', 'application/json', 'application/graphql-response+json', 'text/event-stream', 'multipart/mixed'.",
              "extensions": {
                "code": "BAD_REQUEST"
              }
            }
          ]
        }
        "###);
        assert_eq!(status, 406);
    })
}

#[test]
fn one_valid_acccept_header() {
    runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(FakeGithubSchema).build().await;

        let response = engine
            .raw_execute(
                http::Request::builder()
                    .uri("http://localhost/graphql")
                    .method(http::Method::POST)
                    .header(http::header::CONTENT_TYPE, APPLICATION_JSON)
                    .header(
                        http::header::ACCEPT,
                        "application/jpeg,image/webp,application/json;q=0.8",
                    )
                    .body(br###"{"query":"{ __typename }"}"###.to_vec())
                    .unwrap(),
            )
            .await;
        let status = response.status();
        assert_eq!(
            response
                .headers()
                .get(http::header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
            Some(APPLICATION_JSON)
        );
        let body: serde_json::Value = serde_json::from_slice(&response.into_body()).unwrap();
        insta::assert_json_snapshot!(body, @r###"
        {
          "data": {
            "__typename": "Query"
          }
        }
        "###);
        assert_eq!(status, 200);
    })
}

// POST
// (..)
// A client MUST indicate the media type of a request body using the Content-Type header as specified in RFC7231.
#[rstest::rstest]
#[case::json(APPLICATION_JSON)]
#[case::gql_json(APPLICATION_GRAPHQL_RESPONSE_JSON)]
fn missing_content_type(#[case] accept: &'static str) {
    runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(FakeGithubSchema).build().await;

        let response = engine
            .raw_execute(
                http::Request::builder()
                    .uri("http://localhost/graphql")
                    .method(http::Method::POST)
                    .header(http::header::ACCEPT, accept)
                    .body(br###"{"query":"__typename"}"###.to_vec())
                    .unwrap(),
            )
            .await;
        let status = response.status();
        let body: serde_json::Value = serde_json::from_slice(&response.into_body()).unwrap();
        insta::assert_json_snapshot!(body, @r###"
        {
          "errors": [
            {
              "message": "Missing or invalid Content-Type header. Only 'application/json' is supported.",
              "extensions": {
                "code": "BAD_REQUEST"
              }
            }
          ]
        }
        "###);
        assert_eq!(status, 415);
    })
}

// POST
#[rstest::rstest]
#[case::json(APPLICATION_JSON)]
#[case::gql_json(APPLICATION_GRAPHQL_RESPONSE_JSON)]
fn content_type_with_parameters(#[case] accept: &'static str) {
    runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(FakeGithubSchema).build().await;

        let response = engine
            .raw_execute(
                http::Request::builder()
                    .uri("http://localhost/graphql")
                    .method(http::Method::POST)
                    .header(http::header::ACCEPT, accept)
                    .header(http::header::CONTENT_TYPE, "application/json; charset=utf-8")
                    .body(br###"{"query":"__typename"}"###.to_vec())
                    .unwrap(),
            )
            .await;
        let status = response.status();
        let body: serde_json::Value = serde_json::from_slice(&response.into_body()).unwrap();
        insta::assert_json_snapshot!(body, @r###"
        {
          "errors": [
            {
              "message": " --> 1:1\n  |\n1 | __typename\n  | ^---\n  |\n  = expected executable_definition",
              "locations": [
                {
                  "line": 1,
                  "column": 1
                }
              ],
              "extensions": {
                "code": "OPERATION_PARSING_ERROR"
              }
            }
          ]
        }
        "###);
        assert_ne!(status, 405);
    })
}

// GET requests MUST NOT be used for executing mutation operations. If the values of {query} and {operationName}
// indicate that a mutation operation is to be executed, the server MUST respond with error status code 405 (Method Not Allowed)
// and halt execution. This restriction is necessary to conform with the long-established semantics of safe methods within HTTP.
#[rstest::rstest]
#[case::json(APPLICATION_JSON)]
#[case::gql_json(APPLICATION_GRAPHQL_RESPONSE_JSON)]
fn get_must_not_be_used_for_mutations(#[case] accept: &'static str) {
    runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(Stateful::default()).build().await;

        // Query should work
        let response = engine.get("query { value }").header(http::header::ACCEPT, accept).await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "value": 0
          }
        }
        "###);
        assert_eq!(response.status, 200);

        let response = engine
            .get("mutation { set(val: 1) }")
            .header(http::header::ACCEPT, accept)
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "errors": [
            {
              "message": "Mutation is not allowed with a safe method like GET",
              "extensions": {
                "code": "BAD_REQUEST"
              }
            }
          ]
        }
        "###);
        assert_eq!(response.status, 405);
    })
}

// GET requests MUST NOT be used for executing mutation operations. If the values of {query} and {operationName}
// indicate that a mutation operation is to be executed, the server MUST respond with error status code 405 (Method Not Allowed)
// and halt execution. This restriction is necessary to conform with the long-established semantics of safe methods within HTTP.
#[rstest::rstest]
#[case::json(APPLICATION_JSON)]
#[case::gql_json(APPLICATION_GRAPHQL_RESPONSE_JSON)]
fn get_must_not_be_used_for_mutations_with_sse(#[case] accept: &'static str) {
    runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(Stateful::default()).build().await;

        let accept = format!("text/event-stream,{accept};q=0.9");

        // Query should work
        let response = engine
            .get("query { value }")
            .header(http::header::ACCEPT, accept.clone())
            .into_sse_stream()
            .await;
        insta::assert_json_snapshot!(response.collected_body, @r###"
        [
          {
            "data": {
              "value": 0
            }
          }
        ]
        "###);
        assert_eq!(response.status, 200);

        let response = engine
            .get("mutation { set(val: 1) }")
            .header(http::header::ACCEPT, accept)
            .into_sse_stream()
            .await;
        insta::assert_json_snapshot!(response.collected_body, @r###"
        [
          {
            "errors": [
              {
                "message": "Mutation is not allowed with a safe method like GET",
                "extensions": {
                  "code": "BAD_REQUEST"
                }
              }
            ]
          }
        ]
        "###);
        assert_eq!(response.status, 405);
    })
}
