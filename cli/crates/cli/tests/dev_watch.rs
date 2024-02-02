#![allow(unused_crate_dependencies)]
mod utils;

use std::time::Duration;

use backend::project::GraphType;
use serde_json::Value;
use utils::environment::Environment;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dev_watch() {
    let mut env = Environment::init();

    env.grafbase_init(GraphType::Single);

    env.write_schema(
        r#"
        extend schema @experimental(codegen: true)

        extend type Query {
            hello: String! @resolver(name: "hello")
        }
        "#,
    );
    env.write_resolver("hello.js", "export default function Resolver() { return 'hello'; }");

    env.grafbase_dev_watch();

    let mut client = env.create_client().with_api_key();

    client.poll_endpoint(30, 300).await;

    let response = client.gql::<Value>("query { hello }").send().await;

    let hello: String = dot_get!(response, "data.hello");
    assert_eq!(hello, "hello");

    client.snapshot().await;

    env.write_schema(
        r#"
        extend schema @experimental(codegen: true)

        extend type Query {
            hello: String! @resolver(name: "hello")
            helloAgain: String! @resolver(name: "hello")
        }
        "#,
    );

    client.poll_endpoint_for_changes(30, 300).await;

    let response = client.gql::<Value>("query { helloAgain }").send().await;

    let hello: String = dot_get!(response, "data.helloAgain");
    assert_eq!(hello, "hello");

    // Update the resolver, check that causes changes
    env.write_resolver_async("hello.js", "export default function Resolver() { return 'bye'; }")
        .await;
    // File watcher is on a 1 second debounce so we need to give it a chance to do its thing
    // We're not changing the schema this time so we can't just poll for changes to that
    tokio::time::sleep(Duration::from_secs(10)).await;

    let response = client.gql::<Value>("query { hello helloAgain }").send().await;

    let hello: String = dot_get!(response, "data.hello");
    assert_eq!(hello, "bye");

    let hello: String = dot_get!(response, "data.helloAgain");
    assert_eq!(hello, "bye");

    {
        // Check that the TS resolver types are being generated.
        let generated_types_path = env.directory_path.join("generated/index.ts");
        assert!(generated_types_path.is_file());
    }
}
