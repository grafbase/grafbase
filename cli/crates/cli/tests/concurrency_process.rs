mod utils;

use serde_json::{json, Value};
use utils::consts::{CONCURRENCY_MUTATION, CONCURRENCY_QUERY, CONCURRENCY_SCHEMA};
use utils::environment::Environment;

#[tokio::test]
async fn concurrency_process() {
    let mut env1 = Environment::init(4005);
    let mut env2 = Environment::from(&env1, 4006);

    env1.grafbase_init();
    env1.write_schema(CONCURRENCY_SCHEMA);
    env1.grafbase_dev();

    let async_client1 = env1.create_async_client();
    async_client1.poll_endpoint(30, 300).await;

    env2.grafbase_dev();
    let async_client2 = env2.create_async_client();
    async_client2.poll_endpoint(30, 300).await;

    for _ in 0..15 {
        let (response1, response2): (Value, Value) = tokio::join!(
            async_client1.gql::<Value>(json!({ "query": CONCURRENCY_MUTATION }).to_string()),
            async_client2.gql::<Value>(json!({ "query": CONCURRENCY_MUTATION }).to_string())
        );

        let errors1: Option<Value> = dot_get_opt!(response1, "errors");
        let errors2: Option<Value> = dot_get_opt!(response2, "errors");

        assert!(errors1.is_none(), "errors1: {:#?}", errors1);
        assert!(errors2.is_none(), "errors2: {:#?}", errors2);
    }

    let response1 = async_client1
        .gql::<Value>(json!({ "query": CONCURRENCY_QUERY }).to_string())
        .await;
    let response2 = async_client2
        .gql::<Value>(json!({ "query": CONCURRENCY_QUERY }).to_string())
        .await;

    let result_list1: Vec<Value> = dot_get!(response1, "data.todoListCollection.edges");
    let result_list2: Vec<Value> = dot_get!(response2, "data.todoListCollection.edges");

    assert_eq!(result_list1.len(), 30);
    assert_eq!(result_list2.len(), 30);
}
