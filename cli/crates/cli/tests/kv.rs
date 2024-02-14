#![allow(unused_crate_dependencies)]
mod utils;

use backend::project::GraphType;
use serde_json::Value;
use utils::environment::Environment;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_kv_integration() {
    // prepare
    let mut env = Environment::init();
    env.grafbase_init(GraphType::Single);
    env.write_schema(
        r#"
                extend type Query {
                    hello: String! @resolver(name: "hello")
                    other: String! @resolver(name: "other")
                }
            "#,
    );
    env.write_resolver(
        "hello.js",
        r#"
        export default async function Resolver(_, __, { kv }) {
            const kvKey = "test";

            let { value } = await kv.get(kvKey);
            if (value === null) {
                console.info(`Key ${kvKey} doesn't exist in KV. Creating ...`);
                await kv.set(kvKey, "hello kv!");
            }

            let { value: kv_value } = await kv.get(kvKey);

            return kv_value;
        }
    "#,
    );

    env.write_resolver(
        "other.js",
        r#"
        export default async function Resolver(_, __, { kv }) {
            const kvKey = "test";

            let { value } = await kv.get(kvKey);

            return value ?? "not found";
        }
    "#,
    );

    env.grafbase_dev();
    let client = env
        .create_client_with_options(utils::client::ClientOptionsBuilder::default().http_timeout(60).build())
        .with_api_key();
    client.poll_endpoint(120, 250).await;

    let response = client.gql::<Value>("query { other }").send().await;
    assert_eq!(dot_get!(response, "data.other", String), "not found");

    let response = client.gql::<Value>("query { hello }").send().await;
    assert_eq!(dot_get!(response, "data.hello", String), "hello kv!");

    let response = client.gql::<Value>("query { hello }").send().await;
    assert_eq!(dot_get!(response, "data.hello", String), "hello kv!");

    let response = client.gql::<Value>("query { other }").send().await;
    assert_eq!(dot_get!(response, "data.other", String), "hello kv!");
}
