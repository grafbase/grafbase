use cynic::QueryBuilder;
use cynic_introspection::IntrospectionQuery;
use integration_tests::{mocks::graphql::FakeGithubSchema, runtime, EngineBuilder, MockGraphQlServer, ResponseExt};

#[test]
fn graphql_test_with_transforms() {
    runtime().block_on(async {
        let graphql_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = EngineBuilder::new(schema(graphql_mock.port())).build().await;

        let introspection_query = IntrospectionQuery::build(());
        let response = engine
            .execute(introspection_query)
            .await
            .into_data::<IntrospectionQuery>();

        insta::assert_snapshot!(response.into_schema().unwrap().to_sdl(), @r###"
        input BotInput {
          id: ID!
        }

        scalar CustomRepoId

        type Header {
          name: String!
          value: String!
        }

        type Issue implements PullRequestOrIssue {
          title: String!
        }

        type PullRequest implements PullRequestOrIssue {
          id: ID!
          title: String!
          checks: [String!]!
        }

        interface PullRequestOrIssue {
          title: String!
        }

        input PullRequestsAndIssuesFilters {
          search: String!
        }

        type Query {
          favoriteRepository: CustomRepoId!
          serverVersion: String!
          pullRequestsAndIssues(filter: PullRequestsAndIssuesFilters!): [PullRequestOrIssue!]!
          botPullRequests(bots: [[BotInput!]]!): [PullRequest!]!
          allBotPullRequests: [PullRequest!]!
          pullRequest(id: ID!): PullRequest
          pullRequestOrIssue(id: ID!): PullRequestOrIssue
          headers: [Header!]!
        }

        "###);
    });
}

fn schema(port: u16) -> String {
    format!(
        r#"
          extend schema
          @graphql(
            name: "test",
            namespace: false,
            url: "http://127.0.0.1:{port}",
            schema: "http://127.0.0.1:{port}/spec.json",
            transforms: {{
              exclude: [
                "PullRequest.author",
                "Issue.author"
              ]
            }}
          )
        "#
    )
}
