#![allow(unused_crate_dependencies)]
mod utils;

use backend::project::GraphType;
use serde_json::Value;
use utils::environment::Environment;

use crate::utils::consts::INTROSPECTION_QUERY;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn introspection_configuration() {
    let mut env = Environment::init();

    env.grafbase_init(GraphType::Single);

    env.write_schema_without_introspection(
        r#"
        extend type Query {
            hello: String! @resolver(name: "hello")
        }
        "#,
    );
    env.write_resolver("hello.js", "export default function Resolver() { return 'hello'; }");

    env.grafbase_dev_watch();

    let mut client = env.create_client().with_api_key();

    client.poll_endpoint(30, 300).await;

    let response = client.gql::<Value>(INTROSPECTION_QUERY).send().await;

    let errors: Option<Vec<Value>> = dot_get_opt!(response, "errors");
    assert_eq!(errors, None);

    client.snapshot().await;

    env.write_schema_without_introspection(
        r#"       
        extend schema @introspection(enable: false)
        extend type Query {
            hello: String! @resolver(name: "hello")
        }
        "#,
    );

    client.poll_endpoint_for_changes(30, 300).await;

    let response = client.gql::<Value>(INTROSPECTION_QUERY).send().await;

    insta::assert_json_snapshot!(response, @r###"
    {
      "data": null,
      "errors": [
        {
          "message": "Unauthorized for introspection.",
          "locations": [
            {
              "line": 4,
              "column": 3
            }
          ]
        }
      ]
    }
    "###);

    let errors: Option<Vec<Value>> = dot_get_opt!(response, "errors");
    assert!(errors.is_some_and(|errors| !errors.is_empty()));

    client.snapshot().await;

    env.write_schema_without_introspection(
        r#"       
        extend schema @introspection(enable: true)
        extend type Query {
            hello: String! @resolver(name: "hello")
            helloAgain: String! @resolver(name: "hello")
        }
        "#,
    );

    client.poll_endpoint_for_changes(30, 300).await;

    let response = client.gql::<Value>(INTROSPECTION_QUERY).send().await;

    let errors: Option<Vec<Value>> = dot_get_opt!(response, "errors");
    assert_eq!(errors, None);
}
