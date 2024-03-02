use engine_v2::Engine;
use graphql_mocks::{FakeGithubSchema, MockGraphQlServer};
use integration_tests::{federation::EngineV2Ext, runtime};

#[test]
fn named_fragment_on_object() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::builder()
            .with_schema("schema", &github_mock)
            .await
            .finish()
            .await;

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
            "title": "Creating the thing",
            "checks": [
              "Success!"
            ],
            "author": {
              "name": "Jim"
            }
          },
          {
            "title": "Some bot PR",
            "checks": [
              "Success!"
            ],
            "author": {}
          }
        ]
      }
    }
    "###);
}

#[test]
fn named_fragment_cycle() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::builder()
            .with_schema("schema", &github_mock)
            .await
            .finish()
            .await;

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
                        pullRequests {
                            ...PrFields
                        }
                    }
                    ",
            )
            .await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "errors": [
        {
          "message": "Fragment cycle detected: PrFields, UserFields, PrFields",
          "locations": [
            {
              "line": 19,
              "column": 29
            }
          ]
        }
      ]
    }
    "###);
}

#[test]
fn inline_fragment_on_object() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::builder()
            .with_schema("schema", &github_mock)
            .await
            .finish()
            .await;

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
            "title": "Creating the thing",
            "checks": [
              "Success!"
            ]
          },
          {
            "title": "Some bot PR",
            "checks": [
              "Success!"
            ]
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

        let engine = Engine::builder()
            .with_schema("schema", &github_mock)
            .await
            .finish()
            .await;

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
            "title": "Creating the thing",
            "checks": [
              "Success!"
            ],
            "author": {
              "name": "Jim",
              "email": "jim@example.com"
            }
          },
          {
            "title": "Some bot PR",
            "checks": [
              "Success!"
            ],
            "author": {}
          }
        ]
      }
    }
    "###);
}

#[test]
fn inline_fragments_on_polymorphic_types() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::builder()
            .with_schema("schema", &github_mock)
            .await
            .finish()
            .await;

        engine
            .execute(
                r#"
                    query {
                        pullRequestsAndIssues(filter: { search: "1" }) {
                            ... on PullRequest {
                                __typename
                                title
                                checks
                                author {
                                    __typename
                                    ... on User {
                                        email
                                    }
                                    ... on Bot {
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

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "pullRequestsAndIssues": [
          {
            "__typename": "PullRequest",
            "title": "Creating the thing",
            "checks": [
              "Success!"
            ],
            "author": {
              "__typename": "User",
              "email": "jim@example.com"
            }
          },
          {
            "__typename": "PullRequest",
            "title": "Some bot PR",
            "checks": [
              "Success!"
            ],
            "author": {
              "__typename": "Bot",
              "id": "123"
            }
          },
          {
            "title": "Everythings fine"
          }
        ]
      }
    }
    "###);
}

#[test]
fn named_fragments_on_polymorphic_types() {
    let response = runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::builder()
            .with_schema("schema", &github_mock)
            .await
            .finish()
            .await;

        engine
            .execute(
                r#"
                    query {
                        pullRequestsAndIssues(filter: { search: "1" }) {
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

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "pullRequestsAndIssues": [
          {
            "__typename": "PullRequest",
            "title": "Creating the thing",
            "checks": [
              "Success!"
            ],
            "author": {
              "__typename": "User",
              "email": "jim@example.com"
            }
          },
          {
            "__typename": "PullRequest",
            "title": "Some bot PR",
            "checks": [
              "Success!"
            ],
            "author": {
              "__typename": "Bot",
              "id": "123"
            }
          },
          {
            "author": {
              "email": "pessimist@example.com"
            }
          }
        ]
      }
    }
    "###);
}
