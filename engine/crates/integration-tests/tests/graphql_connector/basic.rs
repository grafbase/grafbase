use integration_tests::{mocks::graphql::FakeGithubSchema, runtime, EngineBuilder, MockGraphQlServer, ResponseExt};
use serde_json::json;

const NAMESPACED_QUERY: &str = "
    query($id: ID!) {
        gothub {
            serverVersion
            pullRequestOrIssue(id: $id) {
                __typename
                title
                ... on GothubPullRequest {
                    checks
                    author {
                        __typename
                        ...AuthorFragment
                    }
                }
                ... on GothubIssue {
                    title
                    author {
                        __typename
                        ...AuthorFragment
                    }
                }
            }
        }
    }

    fragment AuthorFragment on GothubUserOrBot {
        ... on GothubUser {
            email
        }
        ... on GothubBot {
            id
        }
    }
";

#[test]
fn graphql_test_with_namespace() {
    runtime().block_on(async {
        let graphql_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = EngineBuilder::new(schema(graphql_mock.port(), true)).build().await;

        insta::assert_json_snapshot!(
            "namespaced-pull-request-with-user",
            engine
                .execute(NAMESPACED_QUERY)
                .variables(json!({"id": "1"}))
                .await
                .into_value()
        );
        insta::assert_json_snapshot!(
            "namespaced-pull-request-with-bot",
            engine
                .execute(NAMESPACED_QUERY)
                .variables(json!({"id": "2"}))
                .await
                .into_value()
        );
        insta::assert_json_snapshot!(
            "namespaced-issue",
            engine
                .execute(NAMESPACED_QUERY)
                .variables(json!({"id": "3"}))
                .await
                .into_value()
        );
        insta::assert_json_snapshot!(
            "namespaced-null",
            engine
                .execute(NAMESPACED_QUERY)
                .variables(json!({"id": "4"}))
                .await
                .into_value()
        );
        insta::assert_json_snapshot!(
            "namespaced-input-list",
            engine
                .execute(
                    r"
                    query GetPullRequests($bots: [[GothubBotInput!]]!) {
                        gothub {
                            botPullRequests(bots: $bots) {
                                checks
                                author {
                                    __typename
                                }
                            }
                        }
                    }
                "
                )
                .variables(json!({"bots": [[{"id": "2"}]]}))
                .await
                .into_value()
        );
    });
}

const UNNAMESPACED_QUERY: &str = "
    query($id: ID!) {
        serverVersion
        pullRequestOrIssue(id: $id) {
            __typename
            title
            ... on PullRequest {
                checks
                author {
                    __typename
                    ...AuthorFragment
                }
            }
            ... on Issue {
                title
                author {
                    __typename
                    ...AuthorFragment
                }
            }
        }
    }

    fragment AuthorFragment on UserOrBot {
        ... on User {
            email
        }
        ... on Bot {
            id
        }
    }
";

#[test]
fn graphql_test_without_namespace() {
    runtime().block_on(async {
        let graphql_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = EngineBuilder::new(schema(graphql_mock.port(), false)).build().await;

        insta::assert_json_snapshot!(
            "unnamespaced-pull-request-with-user",
            engine
                .execute(UNNAMESPACED_QUERY)
                .variables(json!({"id": "1"}))
                .await
                .into_value()
        );
        insta::assert_json_snapshot!(
            "unnamespaced-pull-request-with-bot",
            engine
                .execute(UNNAMESPACED_QUERY)
                .variables(json!({"id": "2"}))
                .await
                .into_value()
        );
        insta::assert_json_snapshot!(
            "unnamespaced-issue",
            engine
                .execute(UNNAMESPACED_QUERY)
                .variables(json!({"id": "3"}))
                .await
                .into_value()
        );
        insta::assert_json_snapshot!(
            "unnamespaced-null",
            engine
                .execute(UNNAMESPACED_QUERY)
                .variables(json!({"id": "4"}))
                .await
                .into_value()
        );
    });
}

#[test]
fn test_nested_variable_forwarding() {
    runtime().block_on(async {
        let graphql_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = EngineBuilder::new(schema(graphql_mock.port(), false)).build().await;

        engine
            .execute(
                r"
                    query ($search: String!) {
                        pullRequestsAndIssues(filter: {search: $search}) {
                            __typename
                        }
                    }
                ",
            )
            .variables(json!({"search": "1"}))
            .await
            .assert_success();
    });
}

fn schema(port: u16, namespace: bool) -> String {
    format!(
        r#"
          extend schema
          @graphql(
            name: "gothub",
            namespace: {namespace},
            url: "http://127.0.0.1:{port}",
          )
        "#
    )
}
