mod utils;

use json_dotpath::DotPaths;
use serde_json::{json, Value};
use std::env;
use std::thread::sleep;
use std::time::Duration;
use utils::client::Client;
use utils::consts::{DEFAULT_QUERY, DEFAULT_SCHEMA, UPDATED_MUTATION, UPDATED_QUERY, UPDATED_SCHEMA};
use utils::environment::Environment;

#[test]
fn dev_watch() {
    let mut env = Environment::init(4001);

    env.grafbase_init();

    env.write_schema(DEFAULT_SCHEMA);

    env.grafbase_dev_watch();

    let client = Client::new(env.endpoint.clone());

    client.poll_endpoint(30, 300);

    let response = client.gql::<Value>(json!({ "query": DEFAULT_QUERY }).to_string());

    let todo_list_collection: Value = dot_get!(response, "data.todoListCollection.edges");

    assert!(todo_list_collection.is_array());
    assert!(!todo_list_collection.dot_has_checked("<").unwrap());

    env.append_to_schema(UPDATED_SCHEMA);

    // wait for change to be picked up
    if env::var("CI").is_ok() {
        sleep(Duration::from_secs(4));
    } else {
        sleep(Duration::from_secs(2));
    }

    client.poll_endpoint(30, 300);

    client.gql::<Value>(json!({ "query": UPDATED_MUTATION }).to_string());

    let response = client.gql::<Value>(json!({ "query": UPDATED_QUERY }).to_string());
    let author_id: String = dot_get!(response, "data.authorCollection.edges.0.node.id");

    assert!(author_id.starts_with("Author#"));
}
