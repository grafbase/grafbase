use engine_v2::Engine;
use graphql_mocks::FakeGithubSchema;
use integration_tests::{federation::EngineV2Ext, runtime};

#[test]
fn skip_include() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(FakeGithubSchema).build().await;

        engine
            .post(
                r#"
                    query PullRequests($skipping: Boolean! = true) {
                        pullRequestsAndIssues(filter: { search: "1" }) {
                            ...PullRequestFragment @skip(if: $skipping) @include(if: true)
                            ...IssueFragment
                        }
                    }

                    fragment PullRequestFragment on PullRequest {
                        title @skip(if: true)
                        checks @include(if: false)
                        author {
                            ...AuthorFragment
                        }
                    }

                    fragment IssueFragment on Issue {
                        author {
                            ...AuthorFragment
                        }
                    }

                    fragment AuthorFragment on UserOrBot {
                        ... on User @skip(if: $skipping) {
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
          {},
          {},
          {
            "author": {}
          }
        ]
      }
    }
    "###);
}

#[test]
fn skip_include_propagate_between_spreads() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(FakeGithubSchema).build().await;

        engine
            .post(
                r#"
                    query PullRequests {
                        pullRequestsAndIssues(filter: { search: "1" }) {
                            ... on PullRequest @include(if: false) {
                                ... PullRequestFragment @skip(if: false)
                            }
                        }
                    }

                    fragment PullRequestFragment on PullRequest {
                        title
                    }

                    "#,
            )
            .await
    });

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": {
        "pullRequestsAndIssues": [
          {},
          {},
          {}
        ]
      }
    }
    "###);
}

#[test]
fn skip_include_multiple_instances_same_field() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(FakeGithubSchema).build().await;

        engine
            .post(
                r#"
                    query PullRequests {
                        pullRequestsAndIssues(filter: { search: "1" }) {
                            ... on PullRequest @include(if: true) {
                                title
                            }
                            ... on PullRequest @include(if: false) {
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
            "title": "Creating the thing"
          },
          {
            "title": "Some bot PR"
          },
          {}
        ]
      }
    }
    "###);
}
