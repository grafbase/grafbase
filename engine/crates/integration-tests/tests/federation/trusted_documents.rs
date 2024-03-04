use std::collections::HashMap;

use gateway_v2::Gateway;
use graphql_mocks::{FakeGithubSchema, MockGraphQlServer};
use integration_tests::{federation::GatewayV2Ext, runtime};

#[test]
fn trusted_documents() {
    runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;
    });

    let trusted_documents = {
        let mut docs = HashMap::new();
        docs.insert("first-doc-id", "query { serverVersion }");
        docs.insert(
            "favourite-repo-query-doc-id",
            "query { favouriteRepository { owner name } }",
        );
        docs
    };

    let engine = Gateway::builder()
        .with_schema("schema", &github_mock)
        .with_trusted_documents(trusted_documents)
        .await
        .finish()
        .await;
}
