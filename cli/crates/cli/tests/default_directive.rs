#![allow(unused_crate_dependencies)]
mod utils;

use crate::utils::consts::{DEFAULT_DIRECTIVE_CREATE_USER1, DEFAULT_DIRECTIVE_CREATE_USER2, DEFAULT_DIRECTIVE_SCHEMA};
use backend::project::GraphType;
use serde_json::Value;
use utils::environment::Environment;

#[ignore]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn default_directive() {
    let mut env = Environment::init();
    env.grafbase_init(GraphType::Single);
    env.write_schema(DEFAULT_DIRECTIVE_SCHEMA);
    env.grafbase_dev();
    let client = env.create_client().with_api_key();
    client.poll_endpoint(30, 300).await;

    let response = client.gql::<Value>(DEFAULT_DIRECTIVE_CREATE_USER1).send().await;

    let user: serde_json::Value = dot_get!(response, "data.userCreate.user");
    assert_eq!(dot_get!(user, "signInCount", usize), 0);
    assert_eq!(dot_get!(user, "country", String), "Poland");
    assert!(dot_get!(user, "account.active", bool));
    assert_eq!(dot_get!(user, "account.privilege", String), "MEMBER");
    assert_eq!(dot_get!(user, "documents.edges.0.node.name", String), "contract");
    assert_eq!(
        dot_get!(user, "documents.edges.0.node.raw", serde_json::Value),
        serde_json::json!({ "content": "" })
    );

    let response = client.gql::<Value>(DEFAULT_DIRECTIVE_CREATE_USER2).send().await;
    let user: serde_json::Value = dot_get!(response, "data.userCreate.user");
    assert_eq!(dot_get!(user, "signInCount", usize), 1);
    assert_eq!(dot_get!(user, "country", String), "France");
    assert!(dot_get!(user, "account.active", bool));
    assert_eq!(dot_get!(user, "account.privilege", String), "ADMIN");
    assert_eq!(dot_get!(user, "documents.edges.0.node.name", String), "contract");
    assert_eq!(
        dot_get!(user, "documents.edges.0.node.raw", serde_json::Value),
        serde_json::json!({ "key": "value" })
    );
}
