use graphql_mocks::FakeGithubSchema;
use integration_tests::{gateway::Gateway, runtime};
use serde_json::json;

#[test]
fn neighbor_fields() {
    runtime().block_on(async move {
        let engine = Gateway::builder().with_subgraph(FakeGithubSchema).build().await;

        let response = engine
            .post(
                r#"
                    query($filters: PullRequestsAndIssuesFilters!) {
                        pullRequestsAndIssues(filter: $filters) {
                            ... on PullRequest {
                                title
                            }
                        }
                        pullRequestsAndIssues(filter: $filters) {
                            ... on PullRequest {
                                checks
                            }
                        }
                    }
                    "#,
            )
            .variables(json!({"filters": { "search": "1"}}))
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "pullRequestsAndIssues": [
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
              },
              {}
            ]
          }
        }
        "#);
    })
}

#[test]
fn fragment() {
    runtime().block_on(async move {
        let engine = Gateway::builder().with_subgraph(FakeGithubSchema).build().await;

        let response = engine
            .post(
                r#"
                    query($filters: PullRequestsAndIssuesFilters!) {
                        pullRequestsAndIssues(filter: $filters) {
                            ... on PullRequest {
                                title
                            }
                        }
                        ...Test
                    }

                    fragment Test on Query {
                        pullRequestsAndIssues(filter: $filters) {
                            ... on PullRequest {
                                checks
                            }
                        }
                    }
                    "#,
            )
            .variables(json!({"filters": { "search": "1"}}))
            .await;
        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "pullRequestsAndIssues": [
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
              },
              {}
            ]
          }
        }
        "#);
    })
}

#[test]
fn skip_include() {
    runtime().block_on(async move {
        let engine = Gateway::builder().with_subgraph(FakeGithubSchema).build().await;

        let response = engine
            .post(
                r#"
                    query($filters: PullRequestsAndIssuesFilters!, $inc: Boolean!) {
                        pullRequestsAndIssues(filter: $filters) {
                            ... on PullRequest {
                                title
                            }
                        }
                        pullRequestsAndIssues(filter: $filters) @include(if: $inc) {
                            ... on PullRequest {
                                checks
                            }
                        }
                    }
                    "#,
            )
            .variables(json!({"filters": { "search": "1"}, "inc": true}))
            .await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "pullRequestsAndIssues": [
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
              },
              {}
            ]
          }
        }
        "#);
    });
}
