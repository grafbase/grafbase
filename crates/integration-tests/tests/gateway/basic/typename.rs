use graphql_mocks::FakeGithubSchema;
use integration_tests::{gateway::Gateway, runtime};

#[test]
fn typename_alias_should_work() {
    let response = runtime().block_on(async move {
        let engine = Gateway::builder().with_subgraph(FakeGithubSchema::default()).build().await;

        engine
            .post(
                r#"
                query {
                    pullRequestsAndIssues(filter: { search: "1" }) {
                        a: __typename
                        __typename: author { ... on User { email } }
                        ... on PullRequest {
                            __typename: author { ... on User { name } }
                        }
                        ... on Issue {
                            __typename: author { ... on User { name } }
                        }
                    }
                }
                "#,
            )
            .await
    });

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "pullRequestsAndIssues": [
          {
            "a": "PullRequest",
            "__typename": {
              "email": "jim@example.com",
              "name": "Jim"
            }
          },
          {
            "a": "PullRequest",
            "__typename": {}
          },
          {
            "a": "Issue",
            "__typename": {
              "email": "pessimist@example.com",
              "name": "The Pessimist"
            }
          }
        ]
      }
    }
    "#);
}
