#![allow(unused_crate_dependencies)]
mod utils;

use backend::project::GraphType;
use std::collections::HashMap;
use utils::consts::ENVIRONMENT_SCHEMA;
use utils::environment::Environment;

#[ctor::ctor]
fn setup_rustls() {
    rustls::crypto::ring::default_provider().install_default().unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn environment_file() {
    let mut env = Environment::init();

    env.grafbase_init(GraphType::Single);

    env.write_schema(ENVIRONMENT_SCHEMA);

    env.set_variables(HashMap::from([(
        "ISSUER_URL".to_owned(),
        "https://example.com".to_owned(),
    )]));

    env.grafbase_dev();

    let client = env.create_client();

    client.poll_endpoint(30, 300).await;
}

// TODO: add a test for precedence once we have a way to print variables
// (the .env variables are higher priority than process enviroment variables)
