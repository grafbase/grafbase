use gateway_v2::Gateway;
use graphql_mocks::{FakeGithubSchema, MockGraphQlServer};
use integration_tests::{
    federation::{GatewayV2Ext, TestTrustedDocument},
    runtime,
};
use serde_json::json;

#[test]
fn trusted_documents() {
    runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let trusted_documents = vec![
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
                document_text: "query { serverVersion }",
            },
            TestTrustedDocument {
                branch_id: "other-branch-id",
                client_name: "ios-app",
                document_id: "this-one-should-not-be-reachable-on-my-branch",
                document_text: "query { serverVersion }",
            },
        ];

        let engine = Gateway::builder()
            .with_schema("schema", &github_mock)
            .await
            .with_trusted_documents("my-branch-id".to_owned(), trusted_documents.clone())
            .finish()
            .await;

        let execute = |query: &'static str, headers: &[(&str, &str)], extensions: &serde_json::Value| {
            let mut builder = engine.execute(query).extensions(extensions);

            for (header_name, header_value) in headers {
                builder = builder.header(*header_name, *header_value);
            }

            builder
        };

        // Non-trusted-document queries are rejected.
        {
            let response = execute("query { serverVersion }", &[], &serde_json::Value::Null).await;

            insta::assert_json_snapshot!(response, @r###"
            {
              "errors": [
                {
                  "message": "Only trusted document queries are accepted."
                }
              ]
            }
            "###);
        }

        // Trusted document queries without client name header are rejected
        {
            let response = execute(
                "",
                &[],
                &json!({"persistedQuery": { "version": 1, "sha256Hash": &trusted_documents[0].document_id }}),
            )
            .await;

            insta::assert_json_snapshot!(response, @r###"
            {
              "errors": [
                {
                  "message": "Trusted document queries must include the x-graphql-client-name header"
                }
              ]
            }
            "###)
        }

        // Apollo client style happy path
        {
            let response = execute(
                "",
                &[("x-grafbase-client-name", "ios-app")],
                &json!({"persistedQuery": { "version": 1, "sha256Hash": &trusted_documents[0].document_id }}),
            )
            .await;

            insta::assert_json_snapshot!(response, @"")
        }
    });
}
