#![allow(unused_crate_dependencies)]
mod utils;

use utils::environment::Environment;
#[ignore]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[cfg(not(target_os = "windows"))]
async fn start_with_ts_config() {
    let mut env = Environment::init();
    env.set_typescript_config(include_str!("config/default.ts"));
    env.grafbase_start();
    let client = env.create_client().with_api_key();
    client.poll_endpoint(30, 300).await;

    let response = client
        .gql::<serde_json::Value>(
            r"
        query {
            userCollection(first: 100) {
                edges {
                    node {
                        id
                    }
                }
            }
        }
    ",
        )
        .send()
        .await;
    assert_eq!(
        response,
        serde_json::json!({
            "data": {
                "userCollection": {
                    "edges": []
                }
            }
        })
    );
}
