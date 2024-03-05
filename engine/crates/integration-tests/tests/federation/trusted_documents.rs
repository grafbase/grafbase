use futures::Future;
use gateway_v2::Gateway;
use graphql_mocks::{FakeGithubSchema, MockGraphQlServer};
use integration_tests::{
    engine::GraphQlRequest,
    federation::{GatewayV2Ext, TestFederationGateway, TestTrustedDocument},
    runtime,
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
    TestTrustedDocument {
        branch_id: "my-branch-id",
        client_name: "ios-app",
        document_id: r#"
            query { 
                pullRequestsAndIssues(filter: { search: "1" }) { __typename } 
                allBotPullRequests { __typename } 
            }"#,
        document_text: "",
    },
    TestTrustedDocument {
        branch_id: "other-branch-id",
        client_name: "ios-app",
        document_id: "this-one-should-not-be-reachable-on-my-branch",
        document_text: "query { serverVersion }",
    },
];

fn test<Fn, Fut>(test_fn: Fn)
where
    Fn: FnOnce(TestFederationGateway) -> Fut,
    Fut: Future<Output = ()>,
{
    runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Gateway::builder()
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
        let response = engine
            .execute(GraphQlRequest {
                query: String::new(),
                operation_name: None,
                variables: None,
                extensions: None,
                doc_id: Some(TRUSTED_DOCUMENTS[1].document_id.to_owned()),
            })
            .header("x-grafbase-client-name", "ios-app")
            .await;

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "__typename": "Query"
          }
        }
        "###)
    })
}

#[test]
fn apollo_client_style_happy_path() {
    test(|engine| async move {
        let response = engine
            .execute("")
            .extensions(&json!({"persistedQuery": { "version": 1, "sha256Hash": &TRUSTED_DOCUMENTS[0].document_id }}))
            .header("x-grafbase-client-name", "ios-app")
            .await;

        insta::assert_json_snapshot!(response, @r###"
            {
              "data": {
                "serverVersion": "1"
              }
            }
            "###)
    })
}

// #[test]
// fn trusted_documents() {
//     test(|engine| async move {
//         let execute = |query: &'static str, headers: &[(&str, &str)], extensions: &serde_json::Value| {
//             let mut builder = engine.execute(query).extensions(extensions);

//             for (header_name, header_value) in headers {
//                 builder = builder.header(*header_name, *header_value);
//             }

//             builder
//         };

//         // Non-trusted-document queries are rejected.
//         {
//             let response = execute("query { serverVersion }", &[], &serde_json::Value::Null).await;

//             insta::assert_json_snapshot!(response, @r###"
//             {
//               "errors": [
//                 {
//                   "message": "Only trusted document queries are accepted."
//                 }
//               ]
//             }
//             "###);
//         }

//         // Trusted document queries without client name header are rejected
//         {
//             let response = execute(
//                 "",
//                 &[],
//                 &json!({"persistedQuery": { "version": 1, "sha256Hash": &trusted_documents[0].document_id }}),
//             )
//             .await;

//             insta::assert_json_snapshot!(response, @r###"
//             {
//               "errors": [
//                 {
//                   "message": "Trusted document queries must include the x-grafbase-client-name header"
//                 }
//               ]
//             }
//             "###)
//         }

//         // Apollo client style happy path
//         {}

//         // Relay style happy path

//         // TODO test with variables
//     });
// }
