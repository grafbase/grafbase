mod utils;

use utils::consts::ENVIRONMENT_SCHEMA;
use utils::environment::Environment;

#[test]
fn process_environment() {
    let mut env = Environment::init(4013);

    env.grafbase_init();

    env.write_schema(ENVIRONMENT_SCHEMA);

    let dev_output = env.grafbase_dev_output();

    assert!(!dev_output.is_ok());

    std::env::set_var("ISSUER_URL", "https://example.com");

    env.grafbase_dev();

    let client = env.create_client();

    client.poll_endpoint(30, 300);
}

// TODO: add test for precedence once we have a way to print variables
// (the .env variables are higher priority than process enviroment variables)
