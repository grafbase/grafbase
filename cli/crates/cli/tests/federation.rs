#![allow(unused_crate_dependencies, unused_imports, clippy::panic)]
mod utils;

use backend::project::GraphType;
use futures_util::StreamExt;
use graphql_mocks::MockGraphQlServer;
use reqwest_eventsource::RequestBuilderExt;
use serde_json::{json, Value};
use utils::environment::Environment;

#[ctor::ctor]
fn setup_rustls() {
    rustls::crypto::ring::default_provider().install_default().unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
async fn federation_start() {
    use duct::cmd;

    let mut env = Environment::init();
    let output = env.grafbase_init_output(GraphType::Federated);
    assert!(output.status.success());

    let output = cmd!("npm", "install").dir(&env.directory_path).run().unwrap();
    assert!(output.status.success());

    env.grafbase_start();
    let client = env.create_client();
    client.poll_endpoint(30, 300).await;

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
        .send()
        .await;
    insta::assert_json_snapshot!(response, @r###"
    {
      "errors": [
        {
          "message": "there are no subgraphs registered currently",
          "extensions": {
            "code": "INTERNAL_SERVER_ERROR"
          }
        }
      ]
    }
    "###);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[cfg(not(target_os = "windows"))] // tsconfig setup doesn't work on windows :(
async fn test_sse_transport() {
    let mut env = Environment::init_async().await;
    let subscription_server = MockGraphQlServer::new(graphql_mocks::FederatedProductsSchema).await;

    env.grafbase_init(GraphType::Federated);
    env.set_typescript_config(format!(
        r#"
        import {{ config, graph, SubscriptionTransport }} from '@grafbase/sdk'

        export default config({{
            graph: graph.Federated({{
              subscriptions: (subscriptions) => {{
                subscriptions
                  .subgraph("subscriptions")
                  .transport(SubscriptionTransport.GraphQlOverWebsockets, {{
                    url: '{}'
                  }})
              }}
            }}),
        }})
        "#,
        subscription_server.websocket_url(),
    ));
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

    insta::assert_json_snapshot!(events, @r###"
    [
      {
        "data": {
          "newProducts": {
            "upc": "top-4",
            "name": "Jeans",
            "price": 44
          }
        }
      },
      {
        "data": {
          "newProducts": {
            "upc": "top-5",
            "name": "Pink Jeans",
            "price": 55
          }
        }
      }
    ]
    "###);
}

#[tokio::test(flavor = "multi_thread")]
#[cfg(not(target_os = "windows"))] // tsconfig setup doesn't work on windows :(
async fn test_sse_transport_with_auth() {
    let mut env = Environment::init_async().await;
    let identity_server = utils::IdentityServer::new().await;
    let subscription_server = MockGraphQlServer::new(graphql_mocks::FederatedProductsSchema).await;

    env.grafbase_init(GraphType::Federated);
    env.set_typescript_config(format!(
        r#"
        import {{ auth, config, graph, SubscriptionTransport }} from '@grafbase/sdk'

        export default config({{
            graph: graph.Federated({{
              subscriptions: (subscriptions) => {{
                subscriptions
                  .subgraph("subscriptions")
                  .transport(SubscriptionTransport.GraphQlOverWebsockets, {{
                    url: '{}'
                  }})
              }}
            }}),
            auth: {{ providers: [{}] }},
        }})
        "#,
        subscription_server.websocket_url(),
        identity_server.ts_auth_provider()
    ));
    env.prepare_ts_config_dependencies();
    env.grafbase_dev_watch();

    let client = env.create_async_client().with_api_key();
    client.poll_endpoint(30, 300).await;
    env.grafbase_publish_dev("subscriptions", subscription_server.url());

    let events = client
        .with_header("Authorization", identity_server.auth_header())
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

    insta::assert_json_snapshot!(events, @r###"
    [
      {
        "data": {
          "newProducts": {
            "upc": "top-4",
            "name": "Jeans",
            "price": 44
          }
        }
      },
      {
        "data": {
          "newProducts": {
            "upc": "top-5",
            "name": "Pink Jeans",
            "price": 55
          }
        }
      }
    ]
    "###);
}

#[tokio::test(flavor = "multi_thread")]
#[cfg(not(target_os = "windows"))] // tsconfig setup doesn't work on windows :(
async fn test_sse_transport_with_failed_auth() {
    let mut env = Environment::init_async().await;
    let identity_server = utils::IdentityServer::new().await;
    let subscription_server = MockGraphQlServer::new(graphql_mocks::FederatedProductsSchema).await;

    env.grafbase_init(GraphType::Federated);
    env.set_typescript_config(format!(
        r#"
        import {{ auth, config, graph, SubscriptionTransport }} from '@grafbase/sdk'

        export default config({{
            graph: graph.Federated({{
              subscriptions: (subscriptions) => {{
                subscriptions
                  .subgraph("subscriptions")
                  .transport(SubscriptionTransport.GraphQlOverWebsockets, {{
                    url: '{}'
                  }})
              }}
            }}),
            auth: {{ providers: [{}] }},
        }})
        "#,
        subscription_server.websocket_url(),
        identity_server.ts_auth_provider()
    ));
    env.prepare_ts_config_dependencies();
    env.grafbase_dev_watch();

    let client = env.create_async_client().with_api_key();
    client.poll_endpoint(30, 300).await;
    env.grafbase_publish_dev("subscriptions", subscription_server.url());

    let events = client
        .with_header(
            "Authorization",
            identity_server.auth_header_with_claims(json!({"iss": "bogus_issuer"})),
        )
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

    insta::assert_json_snapshot!(events, @r###"
    [
      {
        "errors": [
          {
            "message": "Unauthenticated",
            "extensions": {
              "code": "UNAUTHENTICATED"
            }
          }
        ]
      }
    ]
    "###);
}

#[tokio::test(flavor = "multi_thread")]
#[cfg(not(target_os = "windows"))] // tsconfig setup doesn't work on windows :(
async fn test_multipart_transport() {
    let mut env = Environment::init_async().await;
    let subscription_server = MockGraphQlServer::new(graphql_mocks::FederatedProductsSchema).await;

    env.grafbase_init(GraphType::Federated);
    env.set_typescript_config(format!(
        r#"
        import {{ config, graph, SubscriptionTransport }} from '@grafbase/sdk'

        export default config({{
            graph: graph.Federated({{
              subscriptions: (subscriptions) => {{
                subscriptions
                  .subgraph("subscriptions")
                  .transport(SubscriptionTransport.GraphQlOverWebsockets, {{
                    url: '{}'
                  }})
              }}
            }}),
        }})
        "#,
        subscription_server.websocket_url(),
    ));
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
            "upc": "top-4",
            "name": "Jeans",
            "price": 44
          }
        }
      },
      {
        "data": {
          "newProducts": {
            "upc": "top-5",
            "name": "Pink Jeans",
            "price": 55
          }
        }
      }
    ]
    "###);
}

#[tokio::test(flavor = "multi_thread")]
#[cfg(not(target_os = "windows"))] // tsconfig setup doesn't work on windows :(
async fn test_multipart_transport_with_auth() {
    let mut env = Environment::init_async().await;
    let identity_server = utils::IdentityServer::new().await;
    let subscription_server = MockGraphQlServer::new(graphql_mocks::FederatedProductsSchema).await;

    env.grafbase_init(GraphType::Federated);
    env.set_typescript_config(format!(
        r#"
        import {{ auth, config, graph, SubscriptionTransport }} from '@grafbase/sdk'

        export default config({{
            graph: graph.Federated({{
              subscriptions: (subscriptions) => {{
                subscriptions
                  .subgraph("subscriptions")
                  .transport(SubscriptionTransport.GraphQlOverWebsockets, {{
                    url: '{}'
                  }})
              }}
            }}),
            auth: {{ providers: [{}] }},
        }})
        "#,
        subscription_server.websocket_url(),
        identity_server.ts_auth_provider()
    ));
    env.prepare_ts_config_dependencies();
    env.grafbase_dev_watch();

    let client = env.create_async_client().with_api_key();
    client.poll_endpoint(30, 300).await;
    env.grafbase_publish_dev("subscriptions", subscription_server.url());

    let parts = client
        .with_header("Authorization", identity_server.auth_header())
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
            "upc": "top-4",
            "name": "Jeans",
            "price": 44
          }
        }
      },
      {
        "data": {
          "newProducts": {
            "upc": "top-5",
            "name": "Pink Jeans",
            "price": 55
          }
        }
      }
    ]
    "###);
}

#[tokio::test(flavor = "multi_thread")]
#[cfg(not(target_os = "windows"))] // tsconfig setup doesn't work on windows :(
async fn test_multipart_transport_with_bad_auth() {
    let mut env = Environment::init_async().await;
    let identity_server = utils::IdentityServer::new().await;
    let subscription_server = MockGraphQlServer::new(graphql_mocks::FederatedProductsSchema).await;

    env.grafbase_init(GraphType::Federated);
    env.set_typescript_config(format!(
        r#"
        import {{ auth, config, graph, SubscriptionTransport }} from '@grafbase/sdk'

        export default config({{
            graph: graph.Federated({{
              subscriptions: (subscriptions) => {{
                subscriptions
                  .subgraph("subscriptions")
                  .transport(SubscriptionTransport.GraphQlOverWebsockets, {{
                    url: '{}'
                  }})
              }}
            }}),
            auth: {{ providers: [{}] }},
        }})
        "#,
        subscription_server.websocket_url(),
        identity_server.ts_auth_provider()
    ));
    env.prepare_ts_config_dependencies();
    env.grafbase_dev_watch();

    let client = env.create_async_client().with_api_key();
    client.poll_endpoint(30, 300).await;
    env.grafbase_publish_dev("subscriptions", subscription_server.url());

    let parts = client
        .with_header(
            "Authorization",
            identity_server.auth_header_with_claims(json!({"aud": "bad audience"})),
        )
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
        "errors": [
          {
            "message": "Unauthenticated",
            "extensions": {
              "code": "UNAUTHENTICATED"
            }
          }
        ]
      }
    ]
    "###);
}

#[tokio::test(flavor = "multi_thread")]
#[cfg(not(target_os = "windows"))] // tsconfig setup doesn't work on windows :(
async fn test_websocket_transport() {
    let mut env = Environment::init_async().await;
    let subscription_server = MockGraphQlServer::new(graphql_mocks::FederatedProductsSchema).await;

    env.grafbase_init(GraphType::Federated);
    env.set_typescript_config(format!(
        r#"
        import {{ config, graph, SubscriptionTransport }} from '@grafbase/sdk'

        export default config({{
            graph: graph.Federated({{
              subscriptions: (subscriptions) => {{
                subscriptions
                  .subgraph("subscriptions")
                  .transport(SubscriptionTransport.GraphQlOverWebsockets, {{
                    url: '{}'
                  }})
              }}
            }}),
        }})
        "#,
        subscription_server.websocket_url(),
    ));
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
        .unwrap()
        .collect::<Vec<_>>()
        .await;

    insta::assert_json_snapshot!(response, @r###"
    [
      {
        "data": {
          "newProducts": {
            "upc": "top-4",
            "name": "Jeans",
            "price": 44
          }
        }
      },
      {
        "data": {
          "newProducts": {
            "upc": "top-5",
            "name": "Pink Jeans",
            "price": 55
          }
        }
      }
    ]
    "###);
}

#[tokio::test(flavor = "multi_thread")]
#[cfg(not(target_os = "windows"))] // tsconfig setup doesn't work on windows :(
async fn test_websocket_transport_with_auth() {
    let mut env = Environment::init_async().await;
    let identity_server = utils::IdentityServer::new().await;
    let subscription_server = MockGraphQlServer::new(graphql_mocks::FederatedProductsSchema).await;

    env.grafbase_init(GraphType::Federated);
    env.set_typescript_config(format!(
        r#"
        import {{ auth, config, graph, SubscriptionTransport }} from '@grafbase/sdk'

        export default config({{
            graph: graph.Federated({{
              subscriptions: (subscriptions) => {{
                subscriptions
                  .subgraph("subscriptions")
                  .transport(SubscriptionTransport.GraphQlOverWebsockets, {{
                    url: '{}'
                  }})
              }}
            }}),
            auth: {{ providers: [{}] }},
        }})
        "#,
        subscription_server.websocket_url(),
        identity_server.ts_auth_provider()
    ));
    env.prepare_ts_config_dependencies();
    env.grafbase_dev_watch();

    let client = env.create_async_client().with_api_key();
    client.poll_endpoint(30, 300).await;
    env.grafbase_publish_dev("subscriptions", subscription_server.url());

    let response = client
        .with_header("Authorization", identity_server.auth_header())
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
        .unwrap()
        .collect::<Vec<_>>()
        .await;

    insta::assert_json_snapshot!(response, @r###"
    [
      {
        "data": {
          "newProducts": {
            "upc": "top-4",
            "name": "Jeans",
            "price": 44
          }
        }
      },
      {
        "data": {
          "newProducts": {
            "upc": "top-5",
            "name": "Pink Jeans",
            "price": 55
          }
        }
      }
    ]
    "###);
}

#[tokio::test(flavor = "multi_thread")]
#[cfg(not(target_os = "windows"))] // tsconfig setup doesn't work on windows :(
async fn test_websocket_transport_with_bad_auth() {
    let mut env = Environment::init_async().await;
    let identity_server = utils::IdentityServer::new().await;
    let subscription_server = MockGraphQlServer::new(graphql_mocks::FederatedProductsSchema).await;

    env.grafbase_init(GraphType::Federated);
    env.set_typescript_config(format!(
        r#"
        import {{ auth, config, graph, SubscriptionTransport }} from '@grafbase/sdk'

        export default config({{
            graph: graph.Federated({{
              subscriptions: (subscriptions) => {{
                subscriptions
                  .subgraph("subscriptions")
                  .transport(SubscriptionTransport.GraphQlOverWebsockets, {{
                    url: '{}'
                  }})
              }}
            }}),
            auth: {{ providers: [{}] }},
        }})
        "#,
        subscription_server.websocket_url(),
        identity_server.ts_auth_provider()
    ));
    env.prepare_ts_config_dependencies();
    env.grafbase_dev_watch();

    let client = env.create_async_client().with_api_key();
    client.poll_endpoint(30, 300).await;
    env.grafbase_publish_dev("subscriptions", subscription_server.url());

    let result = client
        .with_header("Authorization", "Bearer notevenclosetoajwt")
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
        .await;

    let Err(error) = result else {
        panic!("Websocket connection should have failed but didn't");
    };

    insta::assert_debug_snapshot!(error, @r###"
    Close(
        4403,
        "Forbidden",
    )
    "###);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[cfg(not(target_os = "windows"))] // tsconfig setup doesn't work on windows :(
async fn test_batch_requests() {
    let mut env = Environment::init_async().await;
    let subscription_server = MockGraphQlServer::new(graphql_mocks::FederatedProductsSchema).await;

    env.grafbase_init(GraphType::Federated);
    env.set_typescript_config(
        r#"
        import { config, graph } from '@grafbase/sdk'

        export default config({
            graph: graph.Federated(),
        })
        "#,
    );
    env.prepare_ts_config_dependencies();
    env.grafbase_dev_watch();

    let client = env.create_async_client().with_api_key();
    client.poll_endpoint(30, 300).await;
    env.grafbase_publish_dev("subscriptions", subscription_server.url());

    let results = client
        .batch_gql(["{ topProducts { name } }", "{ topProducts { upc } }"])
        .await;

    insta::assert_json_snapshot!(results, @r###"
    [
      {
        "data": {
          "topProducts": [
            {
              "name": "Trilby"
            },
            {
              "name": "Fedora"
            },
            {
              "name": "Boater"
            },
            {
              "name": "Jeans"
            },
            {
              "name": "Pink Jeans"
            }
          ]
        }
      },
      {
        "data": {
          "topProducts": [
            {
              "upc": "top-1"
            },
            {
              "upc": "top-2"
            },
            {
              "upc": "top-3"
            },
            {
              "upc": "top-4"
            },
            {
              "upc": "top-5"
            }
          ]
        }
      }
    ]
    "###);
}
