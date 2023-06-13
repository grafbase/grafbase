mod utils;

use backend::project::ConfigType;
use utils::environment::Environment;

#[test]
fn reset() {
    let mut env = Environment::init();

    env.grafbase_init(ConfigType::GraphQL);
    env.grafbase_dev();

    let client = env.create_client();

    client.poll_endpoint(30, 300);

    env.kill_processes();

    assert!(env.has_database_directory());

    env.grafbase_reset();

    assert!(!env.has_database_directory());
}
