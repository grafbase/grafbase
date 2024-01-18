#![allow(unused_crate_dependencies, unused_imports)]
mod utils;

use backend::project::GraphType;
use futures_util::StreamExt;
use graphql_mocks::MockGraphQlServer;
use reqwest_eventsource::RequestBuilderExt;
use serde_json::Value;
use utils::environment::Environment;

#[test]
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn federation_start() {
    use duct::cmd;

    let mut env = Environment::init();
    let output = env.grafbase_init_output(GraphType::Federated);
    assert!(output.status.success());

    let output = cmd!("npm", "install").dir(&env.directory_path).run().unwrap();
    assert!(output.status.success());

    env.grafbase_start();
    let client = env.create_client();
    client.poll_endpoint(30, 300);

    let response = client
        .gql::<serde_json::Value>(
            r"
        query {
          __schema {
            types {
              name
            }
          }
        }
    ",
        )
        .send();
    insta::assert_json_snapshot!(response, @r###"
    {
      "errors": [
        {
          "message": "there are no subgraphs registered currently"
        }
      ]
    }
    "###);
}

#[tokio::test(flavor = "multi_thread")]
#[cfg(not(target_os = "windows"))] // tsconfig setup doesn't work on windows :(
async fn test_sse_transport() {
    let mut env = Environment::init_async().await;
    let subscription_server = MockGraphQlServer::new(graphql_mocks::FakeFederationProductsSchema).await;

    env.grafbase_init(GraphType::Federated);
    env.prepare_ts_config_dependencies();
    env.grafbase_dev_watch();

    let client = env.create_async_client().with_api_key();
    client.poll_endpoint(30, 300).await;
    env.grafbase_publish_dev("subscriptions", subscription_server.url());

    let events = client
        .gql::<Value>(
            r"
            subscription {
                newProducts {
                    upc
                    name
                    price
                }
            }
            ",
        )
        .into_sse_stream()
        .collect::<Vec<_>>()
        .await;

    insta::assert_debug_snapshot!(events, @r###"
    [
        Object {
            "data": Object {
                "newProducts": Object {
                    "name": String("Jeans"),
                    "price": Number(44),
                    "upc": String("top-4"),
                },
            },
        },
        Object {
            "data": Object {
                "newProducts": Object {
                    "name": String("Pink Jeans"),
                    "price": Number(55),
                    "upc": String("top-5"),
                },
            },
        },
    ]
    "###);
}

#[tokio::test(flavor = "multi_thread")]
#[cfg(not(target_os = "windows"))] // tsconfig setup doesn't work on windows :(
async fn test_multipart_transport() {
    let mut env = Environment::init_async().await;
    let subscription_server = MockGraphQlServer::new(graphql_mocks::FakeFederationProductsSchema).await;

    env.grafbase_init(GraphType::Federated);
    env.prepare_ts_config_dependencies();
    env.grafbase_dev_watch();

    let client = env.create_async_client().with_api_key();
    client.poll_endpoint(30, 300).await;
    env.grafbase_publish_dev("subscriptions", subscription_server.url());

    let parts = client
        .gql::<Value>(
            r"
            subscription {
                newProducts {
                    upc
                    name
                    price
                }
            }
            ",
        )
        .into_multipart_stream()
        .await
        .collect::<Vec<_>>()
        .await;

    insta::assert_json_snapshot!(parts, @r###"
    [
      {
        "data": {
          "newProducts": {
            "name": "Jeans",
            "price": 44,
            "upc": "top-4"
          }
        }
      },
      {
        "data": {
          "newProducts": {
            "name": "Pink Jeans",
            "price": 55,
            "upc": "top-5"
          }
        }
      }
    ]
    "###);
}

#[tokio::test(flavor = "multi_thread")]
#[cfg(not(target_os = "windows"))] // tsconfig setup doesn't work on windows :(
async fn test_websocket_transport() {
    let mut env = Environment::init_async().await;
    let subscription_server = MockGraphQlServer::new(graphql_mocks::FakeFederationProductsSchema).await;

    env.grafbase_init(GraphType::Federated);
    env.prepare_ts_config_dependencies();
    env.grafbase_dev_watch();

    let client = env.create_async_client().with_api_key();
    client.poll_endpoint(30, 300).await;
    env.grafbase_publish_dev("subscriptions", subscription_server.url());

    let response = client
        .gql::<Value>(
            r"
            subscription {
                newProducts {
                    upc
                    name
                    price
                }
            }
            ",
        )
        .into_websocket_stream()
        .await
        .collect::<Vec<_>>()
        .await;

    insta::assert_json_snapshot!(response, @r###"
    [
      {
        "data": {
          "newProducts": {
            "name": "Jeans",
            "price": 44,
            "upc": "top-4"
          }
        }
      },
      {
        "data": {
          "newProducts": {
            "name": "Pink Jeans",
            "price": 55,
            "upc": "top-5"
          }
        }
      }
    ]
    "###);
}
