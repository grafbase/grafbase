mod utils;

use json_dotpath::DotPaths;
use serde_json::{json, Value};
use utils::consts::{UNIQUE_CREATE_MUTATION, UNIQUE_DELETE_MUTATION, UNIQUE_QUERY, UNIQUE_SCHEMA};
use utils::environment::Environment;

#[test]
fn unique() {
    let mut env = Environment::init(4003);

    env.grafbase_init();

    env.write_schema(UNIQUE_SCHEMA);

    env.grafbase_dev();

    let client = env.create_client();

    client.poll_endpoint(30, 300);

    client.gql::<Value>(json!({ "query": UNIQUE_CREATE_MUTATION, "variables": { "name": "1" } }).to_string());

    let response = client.gql::<Value>(json!({ "query": UNIQUE_QUERY }).to_string());

    let first_author_id: String = dot_get!(response, "data.authorCollection.edges.0.node.id");

    assert!(first_author_id.starts_with("author_"));

    let response =
        client.gql::<Value>(json!({ "query": UNIQUE_CREATE_MUTATION, "variables": { "name": "1" } }).to_string());

    let errors: Option<Value> = response.dot_get("errors").unwrap();

    assert!(errors.is_some());

    let error: String = dot_get!(response, "errors.0.message");

    assert!(error.contains("value"));
    assert!(error.contains("field"));

    let response =
        client.gql::<Value>(json!({ "query": UNIQUE_CREATE_MUTATION, "variables": { "name": "2" } }).to_string());

    let errors: Option<Value> = response.dot_get("errors").unwrap();

    assert!(errors.is_none());

    let response = client
        .gql::<Value>(json!({ "query": UNIQUE_DELETE_MUTATION, "variables": { "id": first_author_id } }).to_string());

    let errors: Option<Value> = response.dot_get("errors").unwrap();

    assert!(errors.is_none());

    let response =
        client.gql::<Value>(json!({ "query": UNIQUE_CREATE_MUTATION, "variables": { "name": "1" } }).to_string());

    let errors: Option<Value> = response.dot_get("errors").unwrap();

    assert!(errors.is_none());

    assert!(errors.is_none());
}
