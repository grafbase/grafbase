use engine_v2::Engine;
use integration_tests::{federation::EngineV2Ext, mocks::graphql::FakeGithubSchema, runtime};

#[test]
#[ignore]
fn named_fragment_on_object() {
    let engine = Engine::build().with_schema("schema", FakeGithubSchema).finish();

    let response = runtime()
        .block_on(async move {
            engine
                .execute(
                    r#"
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
                    "#,
                )
                .await
        })
        .unwrap();

    insta::assert_json_snapshot!(response, @"");
}

#[test]
#[ignore]
fn inline_fragment_on_object() {
    let engine = Engine::build().with_schema("schema", FakeGithubSchema).finish();

    let response = runtime()
        .block_on(async move {
            engine
                .execute(
                    r#"
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
                    "#,
                )
                .await
        })
        .unwrap();

    insta::assert_json_snapshot!(response, @"");
}

#[test]
#[ignore]
fn inline_fragment_on_object_with_type_condition() {
    let engine = Engine::build().with_schema("schema", FakeGithubSchema).finish();

    let response = runtime()
        .block_on(async move {
            engine
                .execute(
                    r#"
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
                    "#,
                )
                .await
        })
        .unwrap();

    insta::assert_json_snapshot!(response, @"");
}

#[test]
#[ignore]
fn inline_fragments_on_polymorphic_types() {
    let engine = Engine::build().with_schema("schema", FakeGithubSchema).finish();

    let response = runtime()
        .block_on(async move {
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
    let engine = Engine::build().with_schema("schema", FakeGithubSchema).finish();

    let response = runtime()
        .block_on(async move {
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
