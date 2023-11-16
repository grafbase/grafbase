#![allow(unused_crate_dependencies)]
mod utils;

use backend::project::GraphType;
use serde_json::Value;
use utils::consts::{
    COMPILATION_ERROR_QUERY, COMPILATION_ERROR_RESOLVER_QUERY, COMPILATION_ERROR_RESOLVER_SCHEMA,
    COMPILATION_ERROR_SCHEMA, DEFAULT_QUERY, DEFAULT_SCHEMA,
};
use utils::environment::Environment;

#[test]
fn compilation_error_schema() {
    let mut env = Environment::init();
    env.grafbase_init(GraphType::Single);
    env.write_schema(COMPILATION_ERROR_SCHEMA);

    env.grafbase_dev_watch();
    let mut client = env.create_client().with_api_key();
    client.poll_endpoint(30, 300);

    let response = client.gql::<Value>(COMPILATION_ERROR_QUERY).send();
    let errors: Option<Vec<Value>> = dot_get_opt!(response, "errors");

    assert_eq!(errors.map(|errors| !errors.is_empty()), Some(true));

    client.snapshot();

    env.write_schema(DEFAULT_SCHEMA);

    client.poll_endpoint_for_changes(30, 300);

    let response = client.gql::<Value>(DEFAULT_QUERY).send();

    let errors: Option<Vec<Value>> = dot_get_opt!(response, "errors");

    assert!(errors.is_none());

    client.snapshot();
}

#[test]
fn compilation_error_resolvers() {
    let mut env = Environment::init();
    env.grafbase_init(GraphType::Single);
    env.write_schema(COMPILATION_ERROR_RESOLVER_SCHEMA);
    env.write_resolver(
        "return-title.js",
        r"
            export xyz {
                return parent.title;
            }
        ",
    );

    // For now without watching before we investigate the issue.
    env.grafbase_dev();

    let client = env
        .create_client_with_options(utils::client::ClientOptionsBuilder::default().http_timeout(60).build())
        .with_api_key();
    client.poll_endpoint(30, 300);

    let response = client.gql::<Value>(COMPILATION_ERROR_RESOLVER_QUERY).send();

    let errors: Option<Vec<Value>> = dot_get_opt!(response, "errors");

    assert_eq!(errors.map(|errors| !errors.is_empty()), Some(true));

    // FIXME: Uncomment after we figure out the weird race with file change modification.
    /*
    client.snapshot();

    env.write_resolver(
        "return-title.js",
        r#"
            export default function Resolver({ parent, args, context, info }) {
                return parent.title;
            }
        "#,
    );

    client.poll_endpoint_for_changes(30, 300);

    let response = client.gql::<Value>(COMPILATION_ERROR_RESOLVER_QUERY).send();

    let errors: Option<Vec<Value>> = dot_get_opt!(response, "errors");

    assert!(errors.is_none());

    client.snapshot();
    */
}
