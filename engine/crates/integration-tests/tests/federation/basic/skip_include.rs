use engine_v2::Engine;
use graphql_mocks::FakeGithubSchema;
use integration_tests::{federation::EngineV2Ext, runtime};
use serde_json::json;

#[test]
fn skip_include() {
    let response = runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(FakeGithubSchema).build().await;

        engine
            .post(
                r#"
                    query PullRequests($skipping: Boolean! = true) {
                        pullRequestsAndIssues(filter: { search: "1" }) {
                            ...PrFragment @skip(if: $skipping) @include(if: true)
                            ...IssueFragment
                        }
                    }

                    fragment PrFragment on PullRequest {
                        title @skip(if: false)
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
            .variables(json!({}))
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
