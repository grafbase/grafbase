use gateway_v2::Gateway;
use graphql_mocks::{FakeGithubSchema, MockGraphQlServer};
use integration_tests::{
    federation::{GatewayV2Ext, TestTrustedDocument},
    runtime,
};

#[test]
fn trusted_documents() {
    runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let trusted_documents = vec![
            TestTrustedDocument {
                branch_id: "my-branch-id",
                client_name: "ios-app",
                document_id: "first-doc-id",
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
            .with_trusted_documents(trusted_documents.clone())
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

            insta::assert_json_snapshot!(response);
        }
    });
}
