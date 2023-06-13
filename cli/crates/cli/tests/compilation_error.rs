mod utils;

use std::collections::HashMap;

use backend::project::ConfigType;
use serde_json::Value;
use utils::consts::{
    COMPILATION_ERROR_QUERY, COMPILATION_ERROR_RESOLVER_MUTATION, COMPILATION_ERROR_RESOLVER_SCHEMA,
    COMPILATION_ERROR_SCHEMA, DEFAULT_QUERY, DEFAULT_SCHEMA, ENVIRONMENT_SCHEMA,
};
use utils::environment::Environment;

#[test]
#[cfg_attr(target_os = "windows", ignore)]
fn compilation_error() {
    let mut env = Environment::init();
    env.grafbase_init(ConfigType::GraphQL);
    env.write_schema(COMPILATION_ERROR_SCHEMA);
    env.grafbase_dev_watch();
    let mut client = env.create_client().with_api_key();
    client.poll_endpoint(30, 300);

    let response = client.gql::<Value>(COMPILATION_ERROR_QUERY).send();
    let errors: Option<Vec<String>> = dot_get_opt!(response, "errors");

    assert_eq!(errors.map(|errors| !errors.is_empty()), Some(true));

    let error_page = client.get_playground_html();

    assert!(error_page.contains("Encountered a compilation error"));

    client.snapshot();

    env.write_schema(DEFAULT_SCHEMA);

    client.poll_endpoint_for_changes(30, 300);

    let response = client.gql::<Value>(DEFAULT_QUERY).send();

    let errors: Option<Vec<String>> = dot_get_opt!(response, "errors");

    assert!(errors.is_none());

    client.snapshot();

    env.write_resolver(
        "return-title.js",
        r#"
            export xyz {
                return parent.title;
            }
        "#,
    );

    env.write_schema(COMPILATION_ERROR_RESOLVER_SCHEMA);

    client.poll_endpoint_for_changes(30, 300);

    let response = client.gql::<Value>(COMPILATION_ERROR_RESOLVER_MUTATION).send();

    let errors: Option<Vec<String>> = dot_get_opt!(response, "errors");

    assert_eq!(errors.map(|errors| !errors.is_empty()), Some(true));

    let error_page = client.get_playground_html();

    assert!(error_page.contains("Encountered a compilation error"));

    client.snapshot();

    env.write_resolver(
        "return-title.js",
        r#"
            export default function Resolver({ parent, args, context, info }) {
                return parent.title;
            }
        "#,
    );

    env.write_schema(COMPILATION_ERROR_RESOLVER_SCHEMA);

    client.poll_endpoint_for_changes(30, 300);

    let response = client.gql::<Value>(COMPILATION_ERROR_RESOLVER_MUTATION).send();

    let errors: Option<Vec<String>> = dot_get_opt!(response, "errors");

    assert!(errors.is_none());

    client.snapshot();

    env.write_schema(ENVIRONMENT_SCHEMA);

    client.poll_endpoint_for_changes(30, 300);

    let error_page = client.get_playground_html();

    assert!(error_page.contains("Encountered a compilation error"));

    client.snapshot();

    env.set_variables(HashMap::from([(
        "ISSUER_URL".to_owned(),
        "https://example.com".to_owned(),
    )]));

    client.poll_endpoint_for_changes(30, 300);

    let error_page = client.get_playground_html();

    assert!(!error_page.contains("Encountered a compilation error"));
}
