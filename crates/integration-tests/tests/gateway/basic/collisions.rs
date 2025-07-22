use graphql_mocks::{FakeGithubSchema, dynamic::DynamicSchema};
use integration_tests::{gateway::Gateway, runtime};
use serde_json::json;

#[test]
fn neighbor_fields() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema::default())
            .build()
            .await;

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
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema::default())
            .build()
            .await;

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
        let engine = Gateway::builder()
            .with_subgraph(FakeGithubSchema::default())
            .build()
            .await;

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

#[test]
fn variable_with_different_usage() {
    runtime().block_on(async move {
        let engine = Gateway::builder()
            .with_subgraph(
                DynamicSchema::builder(
                    r#"
                    type Query {
                        first(id: ID!): String!
                        second(id: ID): String!
                    }
                "#,
                )
                .with_resolver("Query", "first", json!("1"))
                .with_resolver("Query", "second", json!("2"))
                .into_subgraph("x"),
            )
            .build()
            .await;

        let response = engine
            .post(
                r#"
                query Test($id: ID!) {
                    first(id: $id)
                    second(id: $id)
                }
                "#,
            )
            .variables(json!({"id": "x"}))
            .await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "first": "1",
            "second": "2"
          }
        }
        "#);
    });
}
