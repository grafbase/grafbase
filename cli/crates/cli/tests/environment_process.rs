#![allow(unused_crate_dependencies)]
mod utils;

use backend::project::GraphType;
use utils::consts::ENVIRONMENT_SCHEMA;
use utils::environment::Environment;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn environment_process() {
    let mut env = Environment::init();

    env.grafbase_init(GraphType::Single);

    env.write_schema(ENVIRONMENT_SCHEMA);

    std::env::set_var("ISSUER_URL", "https://example.com");

    env.grafbase_dev();

    let client = env.create_client();

    client.poll_endpoint(30, 300).await;
}

// TODO: add a test for precedence once we have a way to print variables
// (the .env variables are higher priority than process enviroment variables)
