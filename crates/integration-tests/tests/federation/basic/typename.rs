use engine::Engine;
use graphql_mocks::FakeGithubSchema;
use integration_tests::{federation::EngineExt, runtime};

#[test]
fn typename_alias_should_work() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(FakeGithubSchema).build().await;

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
              "name": "Jim",
              "email": "jim@example.com"
            }
          },
          {
            "a": "PullRequest",
            "__typename": {}
          },
          {
            "a": "Issue",
            "__typename": {
              "name": "The Pessimist",
              "email": "pessimist@example.com"
            }
          }
        ]
      }
    }
    "#);
}
