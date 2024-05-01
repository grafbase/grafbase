use std::fmt::Display;

use backend::project::GraphType;
use cynic::{GraphQlResponse, QueryBuilder};
use cynic_introspection::IntrospectionQuery;

use crate::{
    server,
    utils::{async_client::AsyncClient, environment::Environment},
};

#[tokio::test(flavor = "multi_thread")]
async fn graphql_test_with_transforms() {
    let port = server::run().await;

    let mut env = Environment::init_async().await;
    let client = start_grafbase(&mut env, schema(port)).await;

    let introspection_query = IntrospectionQuery::build(());
    let response = client
        .gql::<GraphQlResponse<IntrospectionQuery>>(introspection_query.query)
        .await;

    insta::assert_snapshot!(response.data.unwrap().into_schema().unwrap().to_sdl(), @r###"
    type Header {
      name: String!
      value: String!
    }

    type Issue implements PullRequestOrIssue {
      title: String!
    }

    type PullRequest implements PullRequestOrIssue {
      checks: [String!]!
      title: String!
    }

    interface PullRequestOrIssue {
      title: String!
    }

    type Query {
      headers: [Header!]!
      pullRequestOrIssue(id: ID!): PullRequestOrIssue
      serverVersion: String!
    }

    "###);
}

async fn start_grafbase(env: &mut Environment, schema: impl AsRef<str> + Display) -> AsyncClient {
    env.grafbase_init(GraphType::Standalone);
    env.write_schema(schema);
    env.set_variables([("API_KEY", "BLAH")]);
    env.grafbase_dev_watch();

    let client = env.create_async_client().with_api_key();

    client.poll_endpoint(30, 300).await;

    client
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
