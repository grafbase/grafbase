//! Tests of batched requests

#![allow(unused_crate_dependencies)]
mod utils;

use backend::project::GraphType;
use utils::{async_client::AsyncClient, environment::Environment};

const SCHEMA: &str = r#"
type Todo {
    id: ID!
    title: String
}

extend type Query {
    todoCollection(first: Int!): [Todo!]! @resolver(name: "todos")
}
"#;

#[ctor::ctor]
fn setup_rustls() {
    rustls::crypto::ring::default_provider().install_default().unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn batching() {
    let mut env = Environment::init_async().await;
    let client = start_grafbase(&mut env, SCHEMA).await;

    let response = client
        .batch_gql([
            r#"query { todoCollection(first: 1) { title } }"#,
            r#"query { todoCollection(first: 2) { title } }"#,
        ])
        .await;

    insta::assert_json_snapshot!(response, @r###"
    [
      {
        "data": {
          "todoCollection": [
            {
              "title": "One"
            }
          ]
        }
      },
      {
        "data": {
          "todoCollection": [
            {
              "title": "One"
            },
            {
              "title": "Two"
            }
          ]
        }
      }
    ]
    "###);
}

async fn start_grafbase(env: &mut Environment, schema: impl AsRef<str>) -> AsyncClient {
    env.grafbase_init(GraphType::Single);
    env.write_schema(schema);
    env.set_variables([("API_KEY", "BLAH")]);
    env.grafbase_dev_watch();
    env.write_file(
        "resolvers/todos.js",
        r#"
            export default function Resolver(_, {first}) {
                const data = [
                    {"id": "1", "title": "One"},
                    {"id": "2", "title": "Two"},
                ];
                return data.slice(0, first);
            }
    "#,
    );

    let client = env.create_async_client().with_api_key();

    client.poll_endpoint(30, 300).await;

    client
}
