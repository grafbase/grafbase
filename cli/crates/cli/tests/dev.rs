mod utils;

use serde_json::{json, Value};
use utils::client::Client;
use utils::consts::{DEFAULT_MUTATION, DEFAULT_QUERY, DEFAULT_SCHEMA};
use utils::environment::Environment;

#[test]
fn dev() {
    let mut env = Environment::init(4000);

    env.grafbase_init();

    env.write_schema(DEFAULT_SCHEMA);

    env.grafbase_dev();

    let client = Client::new(env.endpoint.clone());

    // wait for node to be ready
    client.poll_endpoint(30, 300);

    client.gql::<Value>(json!({ "query": DEFAULT_MUTATION }).to_string());

    let response = client.gql::<Value>(json!({ "query": DEFAULT_QUERY }).to_string());

    let todo_list: Value = dot_get!(response, "data.todoListCollection.edges.0.node");

    let todo_list_id: String = dot_get!(todo_list, "id");

    let first_todo_id: String = dot_get!(todo_list, "todos.0.id");

    assert!(todo_list_id.starts_with("TodoList#"));
    assert!(first_todo_id.starts_with("Todo#"));
}
