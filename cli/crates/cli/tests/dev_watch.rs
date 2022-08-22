mod utils;

use json_dotpath::DotPaths;
use serde_json::{json, Value};
use utils::consts::{DEFAULT_QUERY, DEFAULT_SCHEMA, UPDATED_MUTATION, UPDATED_QUERY, UPDATED_SCHEMA};
use utils::environment::Environment;

#[test]
fn dev_watch() {
    let mut env = Environment::init(4001);

    env.grafbase_init();

    env.write_schema(DEFAULT_SCHEMA);

    env.grafbase_dev_watch();

    let mut client = env.create_client();

    client.poll_endpoint(30, 300);

    let response = client.gql::<Value>(json!({ "query": DEFAULT_QUERY }).to_string());

    let todo_list_collection: Value = dot_get!(response, "data.todoListCollection.edges");

    assert!(todo_list_collection.is_array());
    assert!(!todo_list_collection.dot_has_checked("<").unwrap());

    client.snapshot();

    env.append_to_schema(UPDATED_SCHEMA);

    client.poll_endpoint_for_changes(30, 300);

    client.gql::<Value>(json!({ "query": UPDATED_MUTATION }).to_string());

    let response = client.gql::<Value>(json!({ "query": UPDATED_QUERY }).to_string());

    let user_id: String = dot_get!(response, "data.userCollection.edges.0.node.id");
    let user_birthday: String = dot_get!(response, "data.userCollection.edges.0.node.birthday");
    let user_verified: String = dot_get!(response, "data.userCollection.edges.0.node.verified");

    assert!(user_id.starts_with("user_"));
    assert!(user_birthday.ends_with('Z'));
    assert_eq!(user_verified, "VERIFIED");
}
