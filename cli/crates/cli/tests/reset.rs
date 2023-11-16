#![allow(unused_crate_dependencies)]
mod utils;

use crate::utils::consts::DEFAULT_SCHEMA;
use backend::project::GraphType;
use utils::environment::Environment;

#[test]
fn reset() {
    let mut env = Environment::init();

    env.grafbase_init(GraphType::Single);
    env.write_schema(DEFAULT_SCHEMA);
    env.grafbase_dev();

    let client = env.create_client();

    client.poll_endpoint(30, 300);

    env.kill_processes();

    assert!(env.has_database_directory());

    env.grafbase_reset();

    assert!(!env.has_database_directory());
}
