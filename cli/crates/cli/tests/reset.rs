mod utils;

use utils::environment::Environment;

#[test]
fn reset() {
    let mut env = Environment::init(4004);

    env.grafbase_init();
    env.grafbase_dev();

    let client = env.create_client();

    client.poll_endpoint(30, 300);

    env.kill_processes();

    assert!(env.has_dot_grafbase_directory());

    env.grafbase_reset();

    assert!(!env.has_dot_grafbase_directory());
}
