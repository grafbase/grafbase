use engine_v2::Engine;
use integration_tests::{federation::EngineV2Ext, mocks::graphql::FakeGithubSchema, runtime, MockGraphQlServer};

#[test]
fn named_fragment_on_object() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::build().with_schema("schema", &github_mock).await.finish();

        engine
            .execute(
                r"
                    query {
                        allBotPullRequests {
                            ...PrFields
                        }
                    }

                    fragment PrFields on PullRequest {
                        title
                        checks
                        author {
                            ...UserFields
                        }
                    }

                    fragment UserFields on User {
                        name
                    }
                    ",
            )
            .await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "allBotPullRequests": [
          {
            "author": {
              "name": "Jim"
            },
            "checks": [
              "Success!"
            ],
            "title": "Creating the thing"
          },
          {
            "author": {
              "name": null
            },
            "checks": [
              "Success!"
            ],
            "title": "Some bot PR"
          }
        ]
      }
    }
    "###);
}

#[test]
fn inline_fragment_on_object() {
    let response = runtime().block_on(async move {
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
                            }
                        }
                    }
                    ",
            )
            .await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "allBotPullRequests": [
          {
            "checks": [
              "Success!"
            ],
            "title": "Creating the thing"
          },
          {
            "checks": [
              "Success!"
            ],
            "title": "Some bot PR"
          }
        ]
      }
    }
    "###);
}

#[test]
fn inline_fragment_on_object_with_type_condition() {
    let response = runtime().block_on(async move {
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
                                    ... on User {
                                        name
                                        email
                                    }
                                }
                            }
                        }
                    }
                    ",
            )
            .await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "allBotPullRequests": [
          {
            "author": {
              "email": "jim@example.com",
              "name": "Jim"
            },
            "checks": [
              "Success!"
            ],
            "title": "Creating the thing"
          },
          {
            "author": {
              "email": null,
              "name": null
            },
            "checks": [
              "Success!"
            ],
            "title": "Some bot PR"
          }
        ]
      }
    }
    "###);
}

#[test]
#[ignore]
fn inline_fragments_on_polymorphic_types() {
    let response = runtime().block_on(async move {
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
    });

    insta::assert_json_snapshot!(response, @"");
}

#[test]
#[ignore]
fn named_fragments_on_polymorphic_types() {
    let response = runtime().block_on(async move {
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
    });

    insta::assert_json_snapshot!(response, @"");
}
