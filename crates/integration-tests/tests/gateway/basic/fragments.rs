use graphql_mocks::FakeGithubSchema;
use integration_tests::{gateway::Gateway, runtime};

#[test]
fn named_fragment_on_object() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema::default())
            .build()
            .await;

        engine
            .post(
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
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema::default())
            .build()
            .await;

        engine
            .post(
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

    insta::assert_json_snapshot!(response, @r#"
    {
      "errors": [
        {
          "message": "Fragment cycle detected: PrFields, UserFields, PrFields",
          "locations": [
            {
              "line": 19,
              "column": 32
            }
          ],
          "extensions": {
            "code": "OPERATION_VALIDATION_ERROR"
          }
        }
      ]
    }
    "#);
}

#[test]
fn inline_fragment_on_object() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema::default())
            .build()
            .await;

        engine
            .post(
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
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema::default())
            .build()
            .await;

        engine
            .post(
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
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema::default())
            .build()
            .await;

        engine
            .post(
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
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema::default())
            .build()
            .await;

        engine
            .post(
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
