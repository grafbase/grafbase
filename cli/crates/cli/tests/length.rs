mod utils;

use json_dotpath::DotPaths;
use serde_json::{json, Value};
use utils::consts::{LENGTH_CREATE_MUTATION, LENGTH_SCHEMA, LENGTH_UPDATE_MUTATION};
use utils::environment::Environment;

#[test]
fn length() {
    let mut env = Environment::init(4014);

    env.grafbase_init();

    env.write_schema(LENGTH_SCHEMA);

    env.grafbase_dev();

    let client = env.create_client();

    client.poll_endpoint(30, 300);

    let response = client.gql::<Value>(
        json!({ "query": LENGTH_CREATE_MUTATION, "variables": { "name": "hello", "age": 30 } }).to_string(),
    );

    let errors: Option<Value> = response.dot_get("errors").unwrap();
    assert!(errors.is_none());

    let first_author_id: String = dot_get!(response, "data.authorCreate.author.id");

    let response = client
        .gql::<Value>(json!({ "query": LENGTH_CREATE_MUTATION, "variables": { "name": "1", "age": 30 } }).to_string());

    let errors: Option<Value> = response.dot_get("errors").unwrap();
    assert!(errors.is_some());

    let error: String = dot_get!(response, "errors.0.message");

    assert!(error.contains("value"));
    assert!(error.contains("length"));
    assert!(error.contains("short"));

    let response = client.gql::<Value>(
        json!({ "query": LENGTH_CREATE_MUTATION, "variables": { "name": "helloworld!", "age": 30 } }).to_string(),
    );

    let errors: Option<Value> = response.dot_get("errors").unwrap();

    assert!(errors.is_some());

    client.gql::<Value>(
        json!({ "query": LENGTH_UPDATE_MUTATION, "variables": { "id": first_author_id, "name": "helloworld!", "age": 40 } }).to_string(),
    );

    let errors: Option<Value> = response.dot_get("errors").unwrap();

    assert!(error.contains("value"));
    assert!(error.contains("length"));
    assert!(error.contains("short"));

    assert!(errors.is_some());
}
