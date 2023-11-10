use integration_tests::{mocks::graphql::FakeGithubSchema, runtime, EngineBuilder, MockGraphQlServer, ResponseExt};

#[test]
fn test_defer_on_graphql_connector() {
    // Note: this test relies on async-graphql not supporting @defer
    // When that changes we might need to re-think this
    runtime().block_on(async {
        let graphql_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = EngineBuilder::new(schema(graphql_mock.port())).build().await;

        engine
            .execute(
                r#"
                query {
                    pullRequestOrIssue(id: "1") {
                        ... @defer {
                            __typename
                            title
                        }
                    }
                }
                "#,
            )
            .await
            .assert_success();
    });
}

fn schema(port: u16) -> String {
    format!(
        r#"
          extend schema
          @graphql(
            name: "gothub",
            namespace: false
            url: "http://127.0.0.1:{port}",
            schema: "http://127.0.0.1:{port}/spec.json",
          )
        "#
    )
}
