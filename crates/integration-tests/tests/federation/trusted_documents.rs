use engine::Engine;
use futures::Future;
use graphql_mocks::FakeGithubSchema;
use integration_tests::{
    federation::{EngineExt, GraphQlRequest, TestGateway},
    runtime, TestTrustedDocument,
};
use runtime::trusted_documents_client::TrustedDocumentsEnforcementMode;
use serde_json::json;

const TRUSTED_DOCUMENTS: &[TestTrustedDocument] = &[
    TestTrustedDocument {
        branch_id: "my-branch-id",
        client_name: "ios-app",
        document_id: "c6f3443b02e35172b1297e7efbbb1210a48116e363218ad332ee722153105d4a",
        document_text: "query { serverVersion }",
    },
    TestTrustedDocument {
        branch_id: "my-branch-id",
        client_name: "ios-app",
        document_id: "favourite-repo-query-doc-id",
        document_text: "query { __typename }",
    },
];

fn test<Fn, Fut>(enforcement_mode: TrustedDocumentsEnforcementMode, test_fn: Fn)
where
    Fn: FnOnce(TestGateway) -> Fut,
    Fut: Future<Output = ()>,
{
    runtime().block_on(async move {
        let engine = Engine::builder()
            .with_subgraph(FakeGithubSchema)
            .with_mock_trusted_documents(enforcement_mode, TRUSTED_DOCUMENTS.to_owned())
            .build()
            .await;

        test_fn(engine).await
    })
}

#[test]
fn relay_style_happy_path() {
    test(TrustedDocumentsEnforcementMode::Enforce, |engine| async move {
        let send = || {
            engine
                .post(GraphQlRequest {
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
    test(TrustedDocumentsEnforcementMode::Enforce, |engine| async move {
        let send = || {
            engine
                .post("")
                .extensions(
                    json!({"persistedQuery": { "version": 1, "sha256Hash": &TRUSTED_DOCUMENTS[0].document_id }}),
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
fn enforce_mode_regular_non_persisted_queries_without_client_name_are_rejected() {
    test(TrustedDocumentsEnforcementMode::Enforce, |engine| async move {
        let response = engine.post("query { __typename pullRequests { id } }").await;

        insta::assert_json_snapshot!(response, @r#"
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
        "#);
    });
}

#[test]
fn enforce_mode_regular_non_persisted_queries_with_client_name_are_rejected() {
    test(TrustedDocumentsEnforcementMode::Enforce, |engine| async move {
        let response = engine
            .post("query { __typename pullRequests { id } }")
            .header("x-grafbase-client-name", "ios-app")
            .await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "The query document does not match any trusted document. (Unknown trusted document id: '5a6881cd181e6904353b2b627ba351c94c7407771f4e24606526f77808ac42f9')",
              "extensions": {
                "code": "TRUSTED_DOCUMENT_ERROR"
              }
            }
          ]
        }
        "#);
    });
}

#[test]
fn enforce_mode_trusted_document_body_without_document_id() {
    test(TrustedDocumentsEnforcementMode::Enforce, |engine| async move {
        let response = engine
            .post(TRUSTED_DOCUMENTS[0].document_text)
            .header("x-grafbase-client-name", "ios-app")
            .await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "serverVersion": "1"
          }
        }
        "#);
    });
}

#[test]
fn enforce_mode_trusted_document_body_with_matching_document_id() {
    test(TrustedDocumentsEnforcementMode::Enforce, |engine| async move {
        let response = engine
            .post(GraphQlRequest {
                query: TRUSTED_DOCUMENTS[1].document_text.to_owned(),
                operation_name: None,
                variables: None,
                extensions: None,
                doc_id: Some(TRUSTED_DOCUMENTS[1].document_id.to_owned()),
            })
            .header("x-grafbase-client-name", "ios-app")
            .await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "__typename": "Query"
          }
        }
        "#);
    })
}

#[test]
fn enforce_mode_trusted_document_body_with_non_matching_document_id() {
    test(TrustedDocumentsEnforcementMode::Enforce, |engine| async move {
        let response = engine
            .post(GraphQlRequest {
                query: "query { pullRequestsAndIssues(filter: { search: \"1\" }) { __typename } }".to_string(),
                operation_name: None,
                variables: None,
                extensions: None,
                doc_id: Some(TRUSTED_DOCUMENTS[1].document_id.to_owned()),
            })
            .header("x-grafbase-client-name", "ios-app")
            .await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "__typename": "Query"
          }
        }
        "#);
    })
}

#[test]
fn trusted_document_queries_without_client_name_header_are_rejected() {
    test(TrustedDocumentsEnforcementMode::Enforce, |engine| async move {
        let response = engine
            .post("")
            .extensions(json!({"persistedQuery": { "version": 1, "sha256Hash": &TRUSTED_DOCUMENTS[0].document_id }}))
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
    test(TrustedDocumentsEnforcementMode::Enforce, |engine| async move {
        let response = engine
            .post("")
            .extensions(json!({"persistedQuery": { "version": 1, "sha256Hash": &TRUSTED_DOCUMENTS[0].document_id }}))
            .header("x-grafbase-client-name", "android-app")
            .await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "Unknown trusted document id: 'c6f3443b02e35172b1297e7efbbb1210a48116e363218ad332ee722153105d4a'",
              "extensions": {
                "code": "TRUSTED_DOCUMENT_ERROR"
              }
            }
          ]
        }
        "#);
    });
}

#[test]
fn bypass_header() {
    test(TrustedDocumentsEnforcementMode::Enforce, |engine| async move {
        let response = engine
            .post("query { pullRequestsAndIssues(filter: { search: \"1\" }) { __typename } }")
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
            .post("query { pullRequestsAndIssues(filter: { search: \"1\" }) { __typename } }")
            .await;

        insta::assert_json_snapshot!(response, @r#"
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
        "#);
    })
}

#[test]
fn allow_mode_only_inline_document() {
    test(TrustedDocumentsEnforcementMode::Allow, |engine| async move {
        let response = engine
            .post("query { pullRequestsAndIssues(filter: { search: \"1\" }) { __typename } }")
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
    })
}

#[test]
fn allow_mode_both_inline_document_and_document_id() {
    test(TrustedDocumentsEnforcementMode::Allow, |engine| async move {
        let response = engine
            .post(GraphQlRequest {
                query: "query { pullRequestsAndIssues(filter: { search: \"1\" }) { __typename } }".to_string(),
                operation_name: None,
                variables: None,
                extensions: None,
                doc_id: Some(TRUSTED_DOCUMENTS[1].document_id.to_owned()),
            })
            .header("x-grafbase-client-name", "ios-app")
            .await;

        insta::assert_json_snapshot!(response, @r#"
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
        "#);
    })
}

#[test]
fn allow_mode_only_document_id() {
    test(TrustedDocumentsEnforcementMode::Allow, |engine| async move {
        let response = engine
            .post(GraphQlRequest {
                query: String::new(),
                operation_name: None,
                variables: None,
                extensions: None,
                doc_id: Some(TRUSTED_DOCUMENTS[1].document_id.to_owned()),
            })
            .header("x-grafbase-client-name", "ios-app")
            .await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "__typename": "Query"
          }
        }
        "#);
    })
}

#[test]
fn ignore_mode_with_apollo_extension() {
    test(TrustedDocumentsEnforcementMode::Ignore, |engine| async move {
        let response = engine
            .post(pull_requests_query())
            .extensions(json!({"persistedQuery": { "version": 1, "sha256Hash": &TRUSTED_DOCUMENTS[0].document_id }}))
            .header("x-grafbase-client-name", "android-app")
            .await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "Invalid persisted query sha256Hash",
              "extensions": {
                "code": "PERSISTED_QUERY_ERROR"
              }
            }
          ]
        }
        "#);
    })
}

#[test]
fn ignore_mode_with_relay_doc_id() {
    test(TrustedDocumentsEnforcementMode::Ignore, |engine| async move {
        let response = engine
            .post(GraphQlRequest {
                query: pull_requests_query(),
                operation_name: None,
                variables: None,
                extensions: None,
                doc_id: Some(TRUSTED_DOCUMENTS[1].document_id.to_owned()),
            })
            .header("x-grafbase-client-name", "ios-app")
            .await;

        insta::assert_json_snapshot!(response, @r#"
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
        "#);
    })
}

fn pull_requests_query() -> String {
    "query { pullRequestsAndIssues(filter: { search: \"1\" }) { __typename } }".to_owned()
}
