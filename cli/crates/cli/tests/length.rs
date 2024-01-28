#![allow(unused_crate_dependencies)]
mod utils;

use backend::project::GraphType;
use json_dotpath::DotPaths;
use serde_json::{json, Value};
use utils::consts::{LENGTH_CREATE_MUTATION, LENGTH_SCHEMA, LENGTH_UPDATE_MUTATION};
use utils::environment::Environment;

#[ignore]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn length() {
    let mut env = Environment::init();

    env.grafbase_init(GraphType::Single);

    env.write_schema(LENGTH_SCHEMA);

    env.grafbase_dev();

    let client = env.create_client().with_api_key();

    client.poll_endpoint(30, 300).await;

    let response = client
        .gql::<Value>(LENGTH_CREATE_MUTATION)
        .variables(json!({ "name": "hello", "age": 30 }))
        .send()
        .await;

    let errors: Option<Value> = response.dot_get("errors").unwrap();
    assert!(errors.is_none());

    let first_author_id: String = dot_get!(response, "data.authorCreate.author.id");

    let response = client
        .gql::<Value>(LENGTH_CREATE_MUTATION)
        .variables(json!({ "name": "1", "age": 30 }))
        .send()
        .await;

    let errors: Option<Value> = response.dot_get("errors").unwrap();
    assert!(errors.is_some());

    let error: String = dot_get!(response, "errors.0.message");

    assert!(error.contains("value"));
    assert!(error.contains("length"));
    assert!(error.contains("short"));

    let response = client
        .gql::<Value>(LENGTH_CREATE_MUTATION)
        .variables(json!({ "name": "helloworld!", "age": 30 }))
        .send()
        .await;

    let errors: Option<Value> = response.dot_get("errors").unwrap();

    assert!(errors.is_some());

    client
        .gql::<Value>(LENGTH_UPDATE_MUTATION)
        .variables(json!({ "id": first_author_id, "name": "helloworld!", "age": 40 }))
        .send()
        .await;

    let errors: Option<Value> = response.dot_get("errors").unwrap();

    assert!(error.contains("value"));
    assert!(error.contains("length"));
    assert!(error.contains("short"));

    assert!(errors.is_some());
}
