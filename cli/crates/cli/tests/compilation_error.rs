mod utils;

use serde_json::Value;
use utils::consts::{COMPILATION_ERROR_QUERY, COMPILATION_ERROR_SCHEMA, DEFAULT_QUERY, DEFAULT_SCHEMA};
use utils::environment::Environment;

#[tokio::test]
async fn compilation_error() {
    let mut env = Environment::init();
    env.grafbase_init();
    env.write_schema(COMPILATION_ERROR_SCHEMA);
    env.grafbase_dev_watch();
    let mut client = env.create_async_client().with_api_key();
    client.poll_endpoint(30, 300).await;

    let response = client.gql::<Value>(COMPILATION_ERROR_QUERY).await;
    let errors: Option<Vec<String>> = dot_get_opt!(response, "errors");

    assert_eq!(errors.map(|errors| !errors.is_empty()), Some(true));

    let error_page = reqwest::get(format!("http://127.0.0.1:{}", env.port))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert!(error_page.contains("Encountered a compilation error"));

    env.write_schema(DEFAULT_SCHEMA);

    client.snapshot().await;

    client.poll_endpoint_for_changes(30, 300).await;

    let response = client.gql::<Value>(DEFAULT_QUERY).await;

    let errors: Option<Vec<String>> = dot_get_opt!(response, "errors");

    assert!(errors.is_none());
}
