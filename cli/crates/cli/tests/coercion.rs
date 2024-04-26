#![allow(unused_crate_dependencies)]
mod utils;

use backend::project::GraphType;
use serde_json::{json, Value};
use utils::consts::{COERCION_CREATE_DUMMY, COERCION_SCHEMA};
use utils::environment::Environment;

#[ignore]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn coercion() {
    let mut env = Environment::init();
    env.grafbase_init(GraphType::Standalone);
    env.write_schema(COERCION_SCHEMA);
    env.grafbase_dev();
    let client = env.create_client().with_api_key();
    client.poll_endpoint(30, 300).await;

    let coerce = |variables: Value| client.gql::<Value>(COERCION_CREATE_DUMMY).variables(variables).send();

    // Test from the spec
    // https://spec.graphql.org/October2021/#sec-List.Input-Coercion
    for (value, expected) in [
        (json!([1, 2, 3]), Some(vec![1, 2, 3])),
        (json!([1]), Some(vec![1])),
        (json!(1), Some(vec![1])),
        (Value::Null, None),
    ] {
        let result = dot_get_opt!(
            coerce(json!({ "list": value })).await,
            "data.dummyCreate.dummy.list",
            Vec<i32>
        );
        assert_eq!(result, expected, "Input was {value:?}");
    }

    for (value, expected) in [
        (json!([[1], [2, 3]]), Some(vec![vec![1], vec![2, 3]])),
        (json!([1]), Some(vec![vec![1]])),
        (json!(1), Some(vec![vec![1]])),
        (Value::Null, None),
    ] {
        let response = coerce(json!({ "matrix": value })).await;
        let result = dot_get_opt!(response, "data.dummyCreate.dummy.matrix", Vec<Vec<i32>>);
        assert_eq!(result, expected, "Input was {value:?}");
    }

    let response = coerce(serde_json::from_str(r#"{"list": [1, "b", true]}"#).unwrap()).await;
    let message = dot_get!(response, "errors.0.message", String);
    assert!(message.contains("Cannot parse"), "{}", message);

    let response = coerce(json!({"matrix": [1, 2, 3]})).await;
    let message = dot_get!(response, "errors.0.message", String);
    assert!(message.contains("Expected a List"), "{}", message);
}
