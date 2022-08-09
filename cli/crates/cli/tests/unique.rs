mod utils;

use json_dotpath::DotPaths;
use serde_json::{json, Value};
use utils::consts::{UNIQUE_MUTATION, UNIQUE_QUERY, UNIQUE_SCHEMA};
use utils::environment::Environment;

#[test]
fn dev() {
    let mut env = Environment::init(4000);

    env.grafbase_init();

    env.write_schema(UNIQUE_SCHEMA);

    env.grafbase_dev();

    let client = env.create_client();

    client.poll_endpoint(30, 300);

    client.gql::<Value>(json!({ "query": UNIQUE_MUTATION, "variables": { "name": "1" } }).to_string());

    let response = client.gql::<Value>(json!({ "query": UNIQUE_QUERY }).to_string());

    let author_id: String = dot_get!(response, "data.authorCollection.edges.0.node.id");

    assert!(author_id.starts_with("Author#"));

    let response = client.gql::<Value>(json!({ "query": UNIQUE_MUTATION, "variables": { "name": "1" } }).to_string());

    let errors: Option<Value> = response.dot_get("errors").unwrap();

    assert!(errors.is_some());

    let error: String = dot_get!(response, "errors.0.message");

    assert!(error.contains("value"));
    assert!(error.contains("field"));

    let response = client.gql::<Value>(json!({ "query": UNIQUE_MUTATION, "variables": { "name": "2" } }).to_string());

    let errors: Option<Value> = response.dot_get("errors").unwrap();

    assert!(errors.is_none());
}
