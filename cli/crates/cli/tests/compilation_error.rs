#![allow(unused_crate_dependencies)]
mod utils;

use std::time::Duration;

use backend::project::GraphType;
use serde_json::Value;
use utils::environment::Environment;

const SCHEMA: &str = r#"
extend type Query {
    hello: String! @resolver(name: "hello")
}
"#;

#[test]
fn compilation_error_schema() {
    let mut env = Environment::init();
    env.grafbase_init(GraphType::Single);
    env.write_schema("type Xyz e");

    env.grafbase_dev_watch();
    let mut client = env.create_client().with_api_key();
    client.poll_endpoint(30, 300);

    let response = client.gql::<Value>("query { hello }").send();
    let errors: Option<Vec<Value>> = dot_get_opt!(response, "errors");

    assert_eq!(errors.map(|errors| !errors.is_empty()), Some(true));

    client.snapshot();

    env.write_resolver("hello.js", "export default function Resolver() { return 'hello'; }");
    env.write_schema(SCHEMA);

    // the CI for Linux ARM is *extremely* slow to see those changes.
    std::thread::sleep(Duration::from_secs(10));

    client.poll_endpoint_for_changes(30, 300);

    let response = client.gql::<Value>("query { hello }").send();

    let errors: Option<Vec<Value>> = dot_get_opt!(response, "errors");

    assert!(errors.is_none());

    client.snapshot();
}

#[test]
fn post_startup_compilation_error() {
    let mut env = Environment::init();
    env.grafbase_init(GraphType::Single);
    env.write_schema("");

    env.grafbase_dev_watch();
    let mut client = env.create_client().with_api_key();
    client.poll_endpoint(30, 300);

    env.write_schema("type Xyz e");

    client.snapshot();
    client.poll_endpoint(30, 300);

    let response = client.gql::<Value>("query { hello }").send();
    let errors: Option<Vec<Value>> = dot_get_opt!(response, "errors");

    assert_eq!(errors.map(|errors| !errors.is_empty()), Some(true));

    env.write_schema(SCHEMA);

    client.snapshot();
    client.poll_endpoint(30, 300);
}

#[test]
fn compilation_error_resolvers() {
    let mut env = Environment::init();
    env.grafbase_init(GraphType::Single);
    env.write_schema(SCHEMA);
    env.write_resolver(
        "hello.js",
        r"
            export xyz {
                return 'hello';
            }
        ",
    );

    // For now without watching before we investigate the issue.
    env.grafbase_dev_watch();

    let mut client = env
        .create_client_with_options(utils::client::ClientOptionsBuilder::default().http_timeout(60).build())
        .with_api_key();
    client.poll_endpoint(30, 300);

    let response = client.gql::<Value>("query { hello }").send();

    let errors: Option<Vec<Value>> = dot_get_opt!(response, "errors");

    assert_eq!(errors.map(|errors| !errors.is_empty()), Some(true));

    client.snapshot();

    env.write_resolver(
        "hello.js",
        r#"
            export default function Resolver() {
                return "hello";
            }
        "#,
    );

    // File watcher is on a 1 second debounce so we need to give it a chance to do its thing
    // We're not changing the schema this time so we can't just poll for changes to that
    // For some reason it takes a really long time on Linux ARM for this to be taken into account
    // when running all tests.
    std::thread::sleep(Duration::from_secs(10));

    let response = client.gql::<Value>("query { hello }").send();

    let errors: Option<Vec<Value>> = dot_get_opt!(response, "errors");

    assert!(errors.is_none());

    let hello: String = dot_get!(response, "data.hello");
    assert_eq!(hello, "hello");
}
