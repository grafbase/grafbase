#![allow(unused_crate_dependencies)]

use backend::project::ConfigType;
use serde_json::{json, Value};
use utils::async_client::AsyncClient;

use crate::utils::environment::Environment;
#[path = "../utils/mod.rs"]
mod utils;

mod headers;
mod server;

const NAMESPACED_QUERY: &str = "
    query($id: ID!) {
        gothub {
            pullRequestOrIssue(id: $id) {
                __typename
                title
                ... on GothubPullRequest {
                    checks
                    author {
                        __typename
                        ...AuthorFragment
                    }
                }
                ... on GothubIssue {
                    title
                    author {
                        __typename
                        ...AuthorFragment
                    }
                }
            }
        }
    }

    fragment AuthorFragment on GothubUserOrBot {
        ... on GothubUser {
            email
        }
        ... on GothubBot {
            id
        }
    }
";

#[tokio::test(flavor = "multi_thread")]
async fn graphql_test_with_namespace() {
    server::run(54300).await;

    let mut env = Environment::init_async().await;
    let client = start_grafbase(&mut env, schema(54300, true)).await;

    insta::assert_yaml_snapshot!(
        "namespaced-pull-request-with-user",
        client
            .gql::<Value>(NAMESPACED_QUERY)
            .variables(json!({"id": "1"}))
            .await
    );
    insta::assert_yaml_snapshot!(
        "namespaced-pull-request-with-bot",
        client
            .gql::<Value>(NAMESPACED_QUERY)
            .variables(json!({"id": "2"}))
            .await
    );
    insta::assert_yaml_snapshot!(
        "namespaced-issue",
        client
            .gql::<Value>(NAMESPACED_QUERY)
            .variables(json!({"id": "3"}))
            .await
    );
    insta::assert_yaml_snapshot!(
        "namespaced-null",
        client
            .gql::<Value>(NAMESPACED_QUERY)
            .variables(json!({"id": "4"}))
            .await
    );
}

const UNNAMESPACED_QUERY: &str = "
    query($id: ID!) {
        pullRequestOrIssue(id: $id) {
            __typename
            title
            ... on PullRequest {
                checks
                author {
                    __typename
                    ...AuthorFragment
                }
            }
            ... on Issue {
                title
                author {
                    __typename
                    ...AuthorFragment
                }
            }
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
";

#[tokio::test(flavor = "multi_thread")]
async fn graphql_test_without_namespace() {
    server::run(54301).await;

    let mut env = Environment::init_async().await;
    let client = start_grafbase(&mut env, schema(54301, false)).await;

    insta::assert_yaml_snapshot!(
        "unnamespaced-pull-request-with-user",
        client
            .gql::<Value>(UNNAMESPACED_QUERY)
            .variables(json!({"id": "1"}))
            .await
    );
    insta::assert_yaml_snapshot!(
        "unnamespaced-pull-request-with-bot",
        client
            .gql::<Value>(UNNAMESPACED_QUERY)
            .variables(json!({"id": "2"}))
            .await
    );
    insta::assert_yaml_snapshot!(
        "unnamespaced-issue",
        client
            .gql::<Value>(UNNAMESPACED_QUERY)
            .variables(json!({"id": "3"}))
            .await
    );
    insta::assert_yaml_snapshot!(
        "unnamespaced-null",
        client
            .gql::<Value>(UNNAMESPACED_QUERY)
            .variables(json!({"id": "4"}))
            .await
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_header_forwarding() {
    server::run(54302).await;

    let mut env = Environment::init_async().await;
    let client = start_grafbase(&mut env, schema(54302, false)).await;

    insta::assert_yaml_snapshot!(
        "unnamespaced-pull-request-with-user",
        client
            .gql::<Value>(UNNAMESPACED_QUERY)
            .variables(json!({"id": "1"}))
            .await
    );
    insta::assert_yaml_snapshot!(
        "unnamespaced-pull-request-with-bot",
        client
            .gql::<Value>(UNNAMESPACED_QUERY)
            .variables(json!({"id": "2"}))
            .await
    );
    insta::assert_yaml_snapshot!(
        "unnamespaced-issue",
        client
            .gql::<Value>(UNNAMESPACED_QUERY)
            .variables(json!({"id": "3"}))
            .await
    );
    insta::assert_yaml_snapshot!(
        "unnamespaced-null",
        client
            .gql::<Value>(UNNAMESPACED_QUERY)
            .variables(json!({"id": "4"}))
            .await
    );
}

async fn start_grafbase(env: &mut Environment, schema: impl AsRef<str>) -> AsyncClient {
    env.grafbase_init(ConfigType::GraphQL);
    env.write_schema(schema);
    env.set_variables([("API_KEY", "BLAH")]);
    env.grafbase_dev_watch();

    let client = env.create_async_client().with_api_key();

    client.poll_endpoint(30, 300).await;

    client
}

fn schema(port: u16, namespace: bool) -> String {
    let namespace_param = if namespace { "namespace: \"gothub\"" } else { "" };
    format!(
        r#"
          extend schema
          @graphql(
            {namespace_param}
            url: "http://127.0.0.1:{port}",
            schema: "http://127.0.0.1:{port}/spec.json",
          )
        "#
    )
}
