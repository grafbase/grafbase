mod utils;

use crate::utils::consts::{DEFAULT_DIRECTIVE_CREATE_USER1, DEFAULT_DIRECTIVE_CREATE_USER2, DEFAULT_DIRECTIVE_SCHEMA};
use serde_json::Value;
use utils::environment::Environment;

#[test]
fn default_directive() {
    let mut env = Environment::init(4017);
    env.grafbase_init();
    env.write_schema(DEFAULT_DIRECTIVE_SCHEMA);
    env.grafbase_dev();
    let client = env.create_client();
    client.poll_endpoint(30, 300);

    let response = client.gql::<Value>(DEFAULT_DIRECTIVE_CREATE_USER1).send();

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

    let response = client.gql::<Value>(DEFAULT_DIRECTIVE_CREATE_USER2).send();
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
