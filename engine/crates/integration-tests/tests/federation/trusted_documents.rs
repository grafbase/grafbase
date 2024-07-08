use engine_v2::Engine;
use futures::Future;
use graphql_mocks::{FakeGithubSchema, MockGraphQlServer};
use integration_tests::{
    engine_v1::GraphQlRequest,
    federation::{GatewayV2Ext, TestFederationEngine},
    runtime, TestTrustedDocument,
};
use serde_json::json;

const TRUSTED_DOCUMENTS: &[TestTrustedDocument] = &[
    TestTrustedDocument {
        branch_id: "my-branch-id",
        client_name: "ios-app",
        document_id: "df40d7fae090cfec1c7e96d78ffb4087f0421798d96c4c90df3556c7de585dc9",
        document_text: "query { serverVersion }",
    },
    TestTrustedDocument {
        branch_id: "my-branch-id",
        client_name: "ios-app",
        document_id: "favourite-repo-query-doc-id",
        document_text: "query { __typename }",
    },
];

fn test<Fn, Fut>(test_fn: Fn)
where
    Fn: FnOnce(TestFederationEngine) -> Fut,
    Fut: Future<Output = ()>,
{
    runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::builder()
            .with_schema("schema", &github_mock)
            .await
            .with_trusted_documents("my-branch-id".to_owned(), TRUSTED_DOCUMENTS.to_owned())
            .finish()
            .await;

        test_fn(engine).await
    })
}

#[test]
fn relay_style_happy_path() {
    test(|engine| async move {
        let send = || {
            engine
                .execute(GraphQlRequest {
                    query: String::new(),
                    operation_name: None,
                    variables: None,
                    extensions: None,
                    doc_id: Some(TRUSTED_DOCUMENTS[1].document_id.to_owned()),
                })
                .header("x-grafbase-client-name", "ios-app")
        };

        let response = send().await;

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "__typename": "Query"
          }
        }
        "###);

        let second_response = send().await;

        assert_eq!(response.to_string(), second_response.to_string());
    })
}

#[test]
fn apollo_client_style_happy_path() {
    test(|engine| async move {
        let send = || {
            engine
                .execute("")
                .extensions(
                    &json!({"persistedQuery": { "version": 1, "sha256Hash": &TRUSTED_DOCUMENTS[0].document_id }}),
                )
                .header("x-grafbase-client-name", "ios-app")
        };

        let response = send().await;

        insta::assert_json_snapshot!(response, @r###"
            {
              "data": {
                "serverVersion": "1"
              }
            }
            "###);

        let second_response = send().await;

        assert_eq!(response.to_string(), second_response.to_string());
    })
}

#[test]
fn regular_non_persisted_queries_are_rejected() {
    test(|engine| async move {
        let response = engine.execute("query { __typename }").await;

        insta::assert_json_snapshot!(response, @r###"
        {
          "errors": [
            {
              "message": "Cannot execute a trusted document query: missing documentId, doc_id or the persistedQuery extension.",
              "extensions": {
                "code": "TRUSTED_DOCUMENT_ERROR"
              }
            }
          ]
        }
        "###);
    });
}

#[test]
fn trusted_document_queries_without_client_name_header_are_rejected() {
    test(|engine| async move {
        let response = engine
            .execute("")
            .extensions(&json!({"persistedQuery": { "version": 1, "sha256Hash": &TRUSTED_DOCUMENTS[0].document_id }}))
            .await;

        insta::assert_json_snapshot!(response, @r###"
        {
          "errors": [
            {
              "message": "Trusted document queries must include the x-grafbase-client-name header",
              "extensions": {
                "code": "TRUSTED_DOCUMENT_ERROR"
              }
            }
          ]
        }
        "###);
    })
}

#[test]
fn wrong_client_name() {
    test(|engine| async move {
        let response = engine
            .execute("")
            .extensions(&json!({"persistedQuery": { "version": 1, "sha256Hash": &TRUSTED_DOCUMENTS[0].document_id }}))
            .header("x-grafbase-client-name", "android-app")
            .await;

        insta::assert_json_snapshot!(response, @r###"
        {
          "errors": [
            {
              "message": "Unknown document id: 'df40d7fae090cfec1c7e96d78ffb4087f0421798d96c4c90df3556c7de585dc9'",
              "extensions": {
                "code": "TRUSTED_DOCUMENT_ERROR"
              }
            }
          ]
        }
        "###);
    });
}

#[test]
fn bypass_header() {
    test(|engine| async move {
        let response = engine
            .execute("query { pullRequestsAndIssues(filter: { search: \"1\" }) { __typename } }")
            .header("test-bypass-header", "test-bypass-value")
            .await;

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "pullRequestsAndIssues": [
              {
                "__typename": "PullRequest"
              },
              {
                "__typename": "PullRequest"
              },
              {
                "__typename": "Issue"
              }
            ]
          }
        }
        "###);

        // Should never be available even if it's cached by the engine.
        let response = engine
            .execute("query { pullRequestsAndIssues(filter: { search: \"1\" }) { __typename } }")
            .await;

        insta::assert_json_snapshot!(response, @r###"
        {
          "errors": [
            {
              "message": "Cannot execute a trusted document query: missing documentId, doc_id or the persistedQuery extension.",
              "extensions": {
                "code": "TRUSTED_DOCUMENT_ERROR"
              }
            }
          ]
        }
        "###);
    })
}
