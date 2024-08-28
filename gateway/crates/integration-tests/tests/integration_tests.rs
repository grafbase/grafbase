#![allow(unused_crate_dependencies)]

mod telemetry;

use std::{fs, net::SocketAddr, panic::AssertUnwindSafe, sync::Arc, time::Duration};

use duct::cmd;
use futures_util::{Future, FutureExt};
use gateway_integration_tests::mocks::gdn::GdnResponseMock;
use gateway_integration_tests::{
    cargo_bin, listen_address, runtime, Client, CommandHandles, ConfigContent, GatewayBuilder, TestRequest,
};
use indoc::indoc;
use tempfile::tempdir;
use tokio::time::Instant;
use wiremock::{
    matchers::{header, method, path},
    Mock, ResponseTemplate,
};

const ACCESS_TOKEN: &str = "test";

fn with_static_server<'a, F, T>(
    config: impl Into<ConfigContent<'a>>,
    schema: &str,
    path: Option<&str>,
    headers: Option<&'static [(&'static str, &'static str)]>,
    test: T,
) where
    T: FnOnce(Arc<Client>) -> F,
    F: Future<Output = ()>,
{
    GatewayBuilder {
        toml_config: config.into(),
        schema,
        log_level: None,
        client_url_path: path,
        client_headers: headers,
    }
    .run(test)
}

fn with_hybrid_server<F, T>(config: &str, graph_ref: &str, sdl: &str, test: T)
where
    T: FnOnce(Arc<Client>, GdnResponseMock, SocketAddr) -> F,
    F: Future<Output = ()>,
{
    let temp_dir = tempdir().unwrap();

    let config_path = temp_dir.path().join("grafbase.toml");
    fs::write(&config_path, config).unwrap();

    let addr = listen_address();

    let gdn_response = GdnResponseMock::mock(sdl);

    let res = runtime().block_on(async {
        let response = ResponseTemplate::new(200).set_body_string(gdn_response.as_json().to_string());
        let server = wiremock::MockServer::start().await;

        Mock::given(method("GET"))
            .and(path(format!("/graphs/{graph_ref}/current")))
            .and(header("Authorization", format!("Bearer {ACCESS_TOKEN}")))
            .respond_with(response)
            .mount(&server)
            .await;

        let command = cmd!(
            cargo_bin("grafbase-gateway"),
            "--listen-address",
            &addr.to_string(),
            "--config",
            &config_path.to_str().unwrap(),
            "--graph-ref",
            graph_ref,
        )
        .stdout_null()
        .stderr_null()
        .env("GRAFBASE_GDN_URL", format!("http://{}", server.address()))
        .env("GRAFBASE_ACCESS_TOKEN", ACCESS_TOKEN);

        let mut commands = CommandHandles::new();
        commands.push(command.start().unwrap());

        let client = Arc::new(Client::new(format!("http://{addr}/graphql"), commands));

        client.poll_endpoint(30, 300).await;

        let res = AssertUnwindSafe(test(client.clone(), gdn_response, *server.address()))
            .catch_unwind()
            .await;

        client.kill_handles();

        res
    });

    res.unwrap();
}

fn load_schema(name: &str) -> String {
    let path = format!("./tests/schemas/{name}.graphql");
    fs::read_to_string(path).unwrap()
}

async fn introspect(url: &str) -> String {
    grafbase_graphql_introspection::introspect(url, &[("x-api-key", "")])
        .await
        .unwrap_or_default()
}

// This failed when using `format_with()` inside tracing with opentelemetry. Somehow it gets called
// multiple times which isn't supported and panics...
#[test]
fn trace_log_level() {
    let schema = load_schema("big");

    let query = indoc! {r#"
        query Me {
          me {
            id
          }
        }
    "#};

    GatewayBuilder::new(&schema)
        .with_log_level("trace")
        .run(|client| async move {
            let result = client.execute(query).await.into_body();
            let result = serde_json::to_string_pretty(&result).unwrap();

            insta::assert_snapshot!(&result, @r###"
            {
              "data": null,
              "errors": [
                {
                  "message": "Request to subgraph 'accounts' failed with: error sending request",
                  "path": [
                    "me"
                  ],
                  "extensions": {
                    "code": "SUBGRAPH_REQUEST_ERROR"
                  }
                }
              ]
            }
            "###);
        })
}

#[test]
fn no_config() {
    let schema = load_schema("big");

    let query = indoc! {r#"
        query Me {
          me {
            id
          }
        }
    "#};

    with_static_server(ConfigContent(None), &schema, None, None, |client| async move {
        let result = client.execute(query).await.into_body();
        let result = serde_json::to_string_pretty(&result).unwrap();

        insta::assert_snapshot!(&result, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Request to subgraph 'accounts' failed with: error sending request",
              "path": [
                "me"
              ],
              "extensions": {
                "code": "SUBGRAPH_REQUEST_ERROR"
              }
            }
          ]
        }
        "###);
    })
}

#[test]
fn static_schema() {
    let schema = load_schema("big");

    let query = indoc! {r#"
        query Me {
          me {
            id
          }
        }
    "#};

    with_static_server("", &schema, None, None, |client| async move {
        let result = client.execute(query).await.into_body();
        let result = serde_json::to_string_pretty(&result).unwrap();

        insta::assert_snapshot!(&result, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Request to subgraph 'accounts' failed with: error sending request",
              "path": [
                "me"
              ],
              "extensions": {
                "code": "SUBGRAPH_REQUEST_ERROR"
              }
            }
          ]
        }
        "###);
    })
}

#[test]
fn introspect_enabled() {
    let config = indoc! {r#"
        [graph]
        introspection = true
    "#};

    let schema = load_schema("big");

    with_static_server(config, &schema, None, None, |client| async move {
        let result = introspect(client.endpoint()).await;

        insta::assert_snapshot!(&result, @r###"
        type Cart {
          products: [Product!]!
        }

        type Picture {
          url: String!
          width: Int!
          height: Int!
        }

        type Product {
          name: String!
          upc: String!
          price: Int!
          reviews: [Review!]!
        }

        type Query {
          me: User!
          topProducts: [Product!]!
        }

        type Review {
          id: ID!
          body: String!
          pictures: [Picture!]!
          product: Product!
          author: User
        }

        type Subscription {
          newProducts: Product!
        }

        enum Trustworthiness {
          REALLY_TRUSTED
          KINDA_TRUSTED
          NOT_TRUSTED
        }

        type User {
          id: ID!
          username: String!
          profilePicture: Picture
          reviewCount: Int!
          joinedTimestamp: Int!
          cart: Cart!
          reviews: [Review!]!
          trustworthiness: Trustworthiness!
        }
        "###);
    })
}

#[test]
fn introspect_disabled() {
    let config = indoc! {r#"
        [graph]
        introspection = false
    "#};

    let schema = load_schema("big");

    with_static_server(config, &schema, None, None, |client| async move {
        let result = introspect(client.endpoint()).await;
        insta::assert_snapshot!(&result, @r###""###);
    })
}

#[test]
fn custom_path() {
    let config = indoc! {r#"
        [graph]
        path = "/custom"
    "#};

    let schema = load_schema("big");

    let query = indoc! {r#"
        query Me {
          me {
            id
          }
        }
    "#};

    with_static_server(config, &schema, Some("custom"), None, |client| async move {
        let result = client.execute(query).await.into_body();
        let result = serde_json::to_string_pretty(&result).unwrap();

        insta::assert_snapshot!(&result, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Request to subgraph 'accounts' failed with: error sending request",
              "path": [
                "me"
              ],
              "extensions": {
                "code": "SUBGRAPH_REQUEST_ERROR"
              }
            }
          ]
        }
        "###);
    })
}

#[test]
fn csrf_no_header() {
    let config = indoc! {r#"
        [csrf]
        enabled = true
    "#};

    let schema = load_schema("big");

    let query = indoc! {r#"
        query Me {
          me {
            id
          }
        }
    "#};

    with_static_server(config, &schema, Some("custom"), None, |client| async move {
        let response = client.execute(query).await;
        assert_eq!(http::StatusCode::FORBIDDEN, response.status);
    })
}

#[test]
fn csrf_with_header() {
    let config = indoc! {r#"
        [csrf]
        enabled = true
    "#};

    let schema = load_schema("big");

    let query = indoc! {r#"
        query Me {
          me {
            id
          }
        }
    "#};

    let headers = &[("x-grafbase-csrf-protection", "1")];

    with_static_server(config, &schema, None, Some(headers), |client| async move {
        let result = client.execute(query).await.into_body();
        let result = serde_json::to_string_pretty(&result).unwrap();

        insta::assert_snapshot!(&result, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Request to subgraph 'accounts' failed with: error sending request",
              "path": [
                "me"
              ],
              "extensions": {
                "code": "SUBGRAPH_REQUEST_ERROR"
              }
            }
          ]
        }
        "###);
    })
}

#[test]
fn hybrid_graph() {
    let schema = load_schema("big");

    let query = indoc! {r#"
        query Me {
          me {
            id
          }
        }
    "#};

    with_hybrid_server("", "test_graph", &schema, |client, _, _| async move {
        let result = client.execute(query).await.into_body();
        let result = serde_json::to_string_pretty(&result).unwrap();

        insta::assert_snapshot!(&result, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Request to subgraph 'accounts' failed with: error sending request",
              "path": [
                "me"
              ],
              "extensions": {
                "code": "SUBGRAPH_REQUEST_ERROR"
              }
            }
          ]
        }
        "###);
    });
}

#[test]
fn health_default_config() {
    let config = "";
    let schema = load_schema("big");

    with_static_server(config, &schema, None, None, |client| async move {
        let mut url: reqwest::Url = client.endpoint().parse().unwrap();
        url.set_path("/health");

        let response = client.client().get(url).send().await.unwrap();

        assert_eq!(response.status(), 200);

        let body: serde_json::Value = response.json().await.unwrap();

        insta::assert_json_snapshot!(&body, @r###"
        {
          "status": "healthy"
        }
        "###);
    });
}

#[test]
fn health_custom_path() {
    let config = r#"
        [health]
        path = "/gezondheid"
    "#;

    let schema = load_schema("big");

    with_static_server(config, &schema, None, None, |client| async move {
        let mut url: reqwest::Url = client.endpoint().parse().unwrap();
        url.set_path("/health");

        let response = client.client().get(url.clone()).send().await.unwrap();

        assert_eq!(response.status(), 404);
        assert_eq!(response.text().await.unwrap(), "");

        // Now with the configured path
        url.set_path("/gezondheid");

        let response = client.client().get(url).send().await.unwrap();

        assert_eq!(response.status(), 200);

        let body: serde_json::Value = response.json().await.unwrap();

        insta::assert_json_snapshot!(&body, @r###"
        {
          "status": "healthy"
        }
        "###);
    });
}

#[test]
fn health_disabled() {
    let config = r#"
        [health]
        enabled = false
    "#;

    let schema = load_schema("big");

    with_static_server(config, &schema, None, None, |client| async move {
        let mut url: reqwest::Url = client.endpoint().parse().unwrap();
        url.set_path("/health");

        let response = client.client().get(url).send().await.unwrap();

        assert_eq!(response.status(), 404);
        assert_eq!(response.text().await.unwrap(), "");
    });
}

#[test]
fn health_custom_listener() {
    let config = r#"
        [health]
        path = "/gezondheid"
        listen = "0.0.0.0:9668"
    "#;

    let schema = load_schema("big");

    with_static_server(config, &schema, None, None, |client| async move {
        // First check that the health endpoint on the regular socket is not on.
        let mut url: reqwest::Url = client.endpoint().parse().unwrap();
        url.set_path("/health");

        let response = client.client().get(url).send().await.unwrap();

        assert_eq!(response.status(), 404);
        assert_eq!(response.text().await.unwrap(), "");

        // Then check at the configured port.

        let url: reqwest::Url = "http://127.0.0.1:9668/gezondheid".parse().unwrap();
        let response = client.client().get(url).send().await.unwrap();

        assert_eq!(response.status(), 200);

        let body: serde_json::Value = response.json().await.unwrap();

        insta::assert_json_snapshot!(&body, @r###"
        {
          "status": "healthy"
        }
        "###);
    });
}

#[test]
fn global_rate_limiting() {
    let config = indoc! {r#"
        [gateway.rate_limit.global]
        limit = 1
        duration = "1s"
    "#};

    let schema = load_schema("big");

    let query = indoc! {r#"
        query Me {
          me {
            id
          }
        }
    "#};

    with_static_server(config, &schema, None, None, |client| async move {
        expect_rate_limiting(|| client.execute(query)).await;
    })
}

#[test]
fn subgraph_rate_limiting() {
    let config = indoc! {r#"
        [subgraphs.accounts.rate_limit]
        limit = 1
        duration = "1s"
    "#};

    let schema = load_schema("big");

    let query = indoc! {r#"
        query Me {
          me {
            id
          }
        }
    "#};

    with_static_server(config, &schema, None, None, |client| async move {
        expect_rate_limiting(|| client.execute(query)).await
    })
}

#[test]
fn global_redis_rate_limiting() {
    let config = indoc! {r#"
        [gateway.rate_limit]
        storage = "redis"

        [gateway.rate_limit.global]
        limit = 1
        duration = "1s"
    "#};

    let schema = load_schema("big");

    let query = indoc! {r#"
        query Me {
          me {
            id
          }
        }
    "#};

    with_static_server(config, &schema, None, None, |client| async move {
        expect_rate_limiting(|| client.execute(query)).await
    })
}

#[test]
fn subgraph_redis_rate_limiting() {
    let config = indoc! {r#"
        [gateway.rate_limit]
        storage = "redis"

        [subgraphs.accounts.rate_limit]
        limit = 1
        duration = "1s"
    "#};

    let schema = load_schema("big");

    let query = indoc! {r#"
        query Me {
          me {
            id
          }
        }
    "#};

    with_static_server(config, &schema, None, None, |client| async move {
        expect_rate_limiting(|| client.execute(query)).await
    })
}

#[allow(clippy::panic)]
async fn expect_rate_limiting<F>(f: F)
where
    F: Fn() -> TestRequest,
{
    let destiny = Instant::now().checked_add(Duration::from_secs(60)).unwrap();

    loop {
        let request = f();
        let response = request.await.into_body();

        if response["errors"][0]["extensions"]["code"] == "RATE_LIMITED" {
            break;
        }

        if Instant::now().gt(&destiny) {
            panic!("Expected requests to get rate limited ...");
        }
    }
}
