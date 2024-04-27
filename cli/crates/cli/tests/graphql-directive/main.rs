#![allow(unused_crate_dependencies)]

use std::fmt::Display;

use backend::project::GraphType;
use serde_json::{json, Value};
use utils::async_client::AsyncClient;

use crate::utils::environment::Environment;
#[path = "../utils/mod.rs"]
mod utils;

mod headers;
mod server;
mod transforms;

#[ctor::ctor]
fn setup_rustls() {
    rustls::crypto::ring::default_provider().install_default().unwrap();
}

const NAMESPACED_QUERY: &str = "
    query($id: ID!) {
        gothub {
            serverVersion
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
    let port = server::run().await;

    let mut env = Environment::init_async().await;
    let client = start_grafbase(&mut env, schema(port, true)).await;

    let value = client
        .gql::<Value>(NAMESPACED_QUERY)
        .variables(json!({"id": "1"}))
        .await;
    insta::with_settings!({sort_maps => true}, {
        insta::assert_yaml_snapshot!(
            "namespaced-pull-request-with-user",
            value
        );
    });
    let value = client
        .gql::<Value>(NAMESPACED_QUERY)
        .variables(json!({"id": "2"}))
        .await;
    insta::with_settings!({sort_maps => true}, {
        insta::assert_yaml_snapshot!(
            "namespaced-pull-request-with-bot",
            value
        );
    });
    let value = client
        .gql::<Value>(NAMESPACED_QUERY)
        .variables(json!({"id": "3"}))
        .await;
    insta::with_settings!({sort_maps => true}, {
        insta::assert_yaml_snapshot!("namespaced-issue", value);
    });
    let value = client
        .gql::<Value>(NAMESPACED_QUERY)
        .variables(json!({"id": "4"}))
        .await;
    insta::with_settings!({sort_maps => true}, {
        insta::assert_yaml_snapshot!("namespaced-null", value);
    });
}

const UNNAMESPACED_QUERY: &str = "
    query($id: ID!) {
        serverVersion
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
    let port = server::run().await;

    let mut env = Environment::init_async().await;
    let client = start_grafbase(&mut env, schema(port, false)).await;

    let value = client
        .gql::<Value>(UNNAMESPACED_QUERY)
        .variables(json!({"id": "1"}))
        .await;
    insta::with_settings!({sort_maps => true}, {
        insta::assert_yaml_snapshot!("unnamespaced-pull-request-with-user", value);
    });
    let value = client
        .gql::<Value>(UNNAMESPACED_QUERY)
        .variables(json!({"id": "2"}))
        .await;
    insta::with_settings!({sort_maps => true}, {
        insta::assert_yaml_snapshot!("unnamespaced-pull-request-with-bot", value);
    });
    let value = client
        .gql::<Value>(UNNAMESPACED_QUERY)
        .variables(json!({"id": "3"}))
        .await;
    insta::with_settings!({sort_maps => true}, {
        insta::assert_yaml_snapshot!("unnamespaced-issue", value);
    });
    let value = client
        .gql::<Value>(UNNAMESPACED_QUERY)
        .variables(json!({"id": "4"}))
        .await;
    insta::with_settings!({sort_maps => true}, {
        insta::assert_yaml_snapshot!("unnamespaced-null", value);
    });
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

fn schema(port: u16, namespace: bool) -> String {
    format!(
        r#"
          extend schema
          @graphql(
            name: "gothub",
            namespace: {namespace},
            url: "http://127.0.0.1:{port}",
            schema: "http://127.0.0.1:{port}/spec.json",
          )
        "#
    )
}
