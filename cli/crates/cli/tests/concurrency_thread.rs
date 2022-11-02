mod utils;

use serde_json::{json, Value};
use utils::consts::{CONCURRENCY_MUTATION, CONCURRENCY_QUERY, CONCURRENCY_SCHEMA};
use utils::environment::Environment;

#[tokio::test]
async fn concurrency_thread() {
    let mut env = Environment::init(4007);

    env.grafbase_init();
    env.write_schema(CONCURRENCY_SCHEMA);
    env.grafbase_dev();

    // if using multiple clients, has issues in the CI with the connection being reset,
    // also happens locally if using a larger number of requests.
    // due to the CLI not erroring this seems like a configuration issue or due to using multiple clients
    // but
    // TODO: make sure this isn't due to a concurrency issue
    let async_client = env.create_async_client();

    async_client.poll_endpoint(30, 300).await;

    for _ in 0..10 {
        let (response1, response2, response3): (Value, Value, Value) = tokio::join!(
            async_client.gql::<Value>(json!({ "query": CONCURRENCY_MUTATION }).to_string()),
            async_client.gql::<Value>(json!({ "query": CONCURRENCY_MUTATION }).to_string()),
            async_client.gql::<Value>(json!({ "query": CONCURRENCY_MUTATION }).to_string())
        );

        let errors1: Option<Value> = dot_get_opt!(response1, "errors");
        let errors2: Option<Value> = dot_get_opt!(response2, "errors");
        let errors3: Option<Value> = dot_get_opt!(response3, "errors");

        assert!(errors1.is_none());
        assert!(errors2.is_none());
        assert!(errors3.is_none());
    }

    let response1 = async_client
        .gql::<Value>(json!({ "query": CONCURRENCY_QUERY }).to_string())
        .await;
    let response2 = async_client
        .gql::<Value>(json!({ "query": CONCURRENCY_QUERY }).to_string())
        .await;
    let response3 = async_client
        .gql::<Value>(json!({ "query": CONCURRENCY_QUERY }).to_string())
        .await;

    let result_list1: Vec<Value> = dot_get!(response1, "data.todoListCollection.edges");
    let result_list2: Vec<Value> = dot_get!(response2, "data.todoListCollection.edges");
    let result_list3: Vec<Value> = dot_get!(response3, "data.todoListCollection.edges");

    assert_eq!(result_list1.len(), 30);
    assert_eq!(result_list2.len(), 30);
    assert_eq!(result_list3.len(), 30);
}
