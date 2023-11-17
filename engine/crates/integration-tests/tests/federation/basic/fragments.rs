use engine_v2::Engine;
use integration_tests::{federation::EngineV2Ext, mocks::graphql::FakeGithubSchema, runtime, MockGraphQlServer};

#[test]
#[ignore]
fn named_fragment_on_object() {
    let response = runtime()
        .block_on(async move {
            let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

            let engine = Engine::build().with_schema("schema", &github_mock).await.finish();

            engine
                .execute(
                    r"
                    query {
                        allBotPullRequests {
                            ... PrFields
                        }
                    }

                    fragment PrFields on PullRequest {
                        title
                        checks
                        author {
                            name
                            email
                        }
                    }
                    ",
                )
                .await
        })
        .unwrap();

    insta::assert_json_snapshot!(response, @"");
}

#[test]
#[ignore]
fn inline_fragment_on_object() {
    let response = runtime()
        .block_on(async move {
            let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

            let engine = Engine::build().with_schema("schema", &github_mock).await.finish();

            engine
                .execute(
                    r"
                    query {
                        allBotPullRequests {
                            ... {
                                title
                                checks
                                author {
                                    name
                                    email
                                }
                            }
                        }
                    }
                    ",
                )
                .await
        })
        .unwrap();

    insta::assert_json_snapshot!(response, @"");
}

#[test]
#[ignore]
fn inline_fragment_on_object_with_type_condition() {
    let response = runtime()
        .block_on(async move {
            let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

            let engine = Engine::build().with_schema("schema", &github_mock).await.finish();

            engine
                .execute(
                    r"
                    query {
                        allBotPullRequests {
                            ... on PullRequest {
                                title
                                checks
                                author {
                                    name
                                    email
                                }
                            }
                        }
                    }
                    ",
                )
                .await
        })
        .unwrap();

    insta::assert_json_snapshot!(response, @"");
}

#[test]
#[ignore]
fn inline_fragments_on_polymorphic_types() {
    let response = runtime()
        .block_on(async move {
            let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

            let engine = Engine::build().with_schema("schema", &github_mock).await.finish();

            engine
                .execute(
                    r#"
                    query {
                        pullRequestsAndIssues(id: "1") {
                            ... on PullRequest {
                                __typename
                                title
                                checks
                                author {
                                    __typename
                                    ... on User {
                                        email
                                    }
                                    ... on User Bot {
                                        id
                                    }
                                }
                            }
                            ... on Issue {
                                title
                            }
                        }
                    }
                    "#,
                )
                .await
        })
        .unwrap();

    insta::assert_json_snapshot!(response, @"");
}

#[test]
#[ignore]
fn named_fragments_on_polymorphic_types() {
    let response = runtime()
        .block_on(async move {
            let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

            let engine = Engine::build().with_schema("schema", &github_mock).await.finish();

            engine
                .execute(
                    r#"
                    query {
                        pullRequestsAndIssues(id: "1") {
                            ...PrFragment
                            ...IssueFragment
                        }
                    }

                    fragment PrFragment on PullRequest {
                        __typename
                        title
                        checks
                        author {
                            __typename
                            ...AuthorFragment
                        }
                    }

                    fragment IssueFragment on Issue {
                        author {
                            ...AuthorFragment
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
                    "#,
                )
                .await
        })
        .unwrap();

    insta::assert_json_snapshot!(response, @"");
}
