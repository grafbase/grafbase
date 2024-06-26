#![allow(unused_crate_dependencies, clippy::panic)]

mod utils;

use std::fmt::Display;
use std::time::Duration;

use backend::project::GraphType;
use serde_json::Value;
use tokio::time::Instant;
use utils::{async_client::AsyncClient, environment::Environment};

#[ctor::ctor]
fn setup_rustls() {
    rustls::crypto::ring::default_provider().install_default().unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn rate_limiting() {
    let expected: Value = serde_json::from_str(
        r#"{
              "errors": [
                {
                  "message": "Too many requests"
                }
              ]
            }"#,
    )
    .unwrap();

    let mut env = Environment::init_async().await;
    env.write_resolver(
        "post.js",
        r"
        export default function Resolver(parent, args, context, info) {
            return {
                title: (Math.random() + 1).toString(36).substring(7)
            }
        }
    ",
    );
    let client = start_grafbase(
        &mut env,
        r#"
            extend schema
            @rateLimiting(
              rules: [
                {
                  name: "header",
                  condition: {
                    headers: [
                        {
                            name: "test",
                            value: "*"
                        }
                    ]
                  },
                  limit: 1000,
                  duration: 10
                }
              ]
            )

            type Query {
                post: Post! @resolver(name: "post")
            }

            type Post {
                title: String!
            }
        "#,
    )
    .await;

    let call = || async {
        let response = client
            .gql::<Value>("query { post { title } }")
            .header("test", "any_value")
            .into_reqwest_builder()
            .send()
            .await
            .unwrap();
        (response.headers().clone(), response.json::<Value>().await.unwrap())
    };

    let destiny = Instant::now().checked_add(Duration::from_secs(60)).unwrap();

    loop {
        let response = Box::pin(call());
        let (_, content) = response.await;

        if content.eq(&expected) {
            break;
        }

        if Instant::now().gt(&destiny) {
            panic!("Expected requests to get rate limited ...");
        }
    }
}

async fn start_grafbase(env: &mut Environment, schema: impl AsRef<str> + Display) -> AsyncClient {
    env.grafbase_init(GraphType::Standalone);
    env.write_schema(schema);
    env.grafbase_dev_watch();

    let client = env.create_async_client().with_api_key();

    client.poll_endpoint(30, 300).await;

    client
}
