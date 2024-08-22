use cynic::QueryBuilder;
use cynic_introspection::IntrospectionQuery;
use graphql_mocks::{FakeGithubSchema, MockGraphQlServer};
use integration_tests::{runtime, EngineBuilder, ResponseExt};

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
          sentient: Boolean! = false
        }

        scalar CustomRepoId

        type Issue implements PullRequestOrIssue {
          title: String!
        }

        type PullRequest implements PullRequestOrIssue {
          checks: [String!]!
          id: ID!
          status: Status!
          title: String!
        }

        interface PullRequestOrIssue {
          title: String!
        }

        input PullRequestsAndIssuesFilters {
          search: String!
        }

        type Query {
          allBotPullRequests: [PullRequest!]!
          botPullRequests(bots: [[BotInput!]]!): [PullRequest!]!
          fail: Int!
          favoriteRepository: CustomRepoId!
          pullRequest(id: ID!): PullRequest
          pullRequestOrIssue(id: ID!): PullRequestOrIssue
          pullRequestsAndIssues(filter: PullRequestsAndIssuesFilters!): [PullRequestOrIssue!]!
          serverVersion: String!
          sillyDefaultValue(status: Status! = OPEN): String!
          statusString(status: Status!): String!
        }

        enum Status {
          CLOSED
          OPEN
        }

        "###);
    });
}

fn schema(port: u16) -> String {
    format!(
        r#"
          extend schema @introspection(enable: true)
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
