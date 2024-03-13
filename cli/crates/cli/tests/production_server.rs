#![allow(unused_crate_dependencies)]

use std::{
    env, fs,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    panic::{catch_unwind, AssertUnwindSafe},
    process::Output,
    sync::{Arc, OnceLock},
};

use duct::cmd;
use futures_util::{Future, FutureExt};
use indoc::indoc;
use tempfile::tempdir;
use tokio::runtime::Runtime;
use utils::{
    client::ClientOptions,
    environment::{get_free_port, CommandHandles},
};
use wiremock::{
    matchers::{header, method, path},
    Mock, ResponseTemplate,
};

use crate::utils::{cargo_bin::cargo_bin, client::Client};

mod utils;

const ACCESS_TOKEN: &str = "test";

pub fn runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
    })
}

fn listen_address() -> SocketAddr {
    let port = get_free_port();

    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port))
}

fn with_static_server<F, T>(
    config: &str,
    schema: &str,
    path: Option<&str>,
    headers: Option<&'static [(&'static str, &'static str)]>,
    test: T,
) where
    T: FnOnce(Arc<Client>) -> F,
    F: Future<Output = ()>,
{
    let temp_dir = tempdir().unwrap();

    let config_path = temp_dir.path().join("grafbase.toml");
    fs::write(&config_path, config).unwrap();

    let schema_path = temp_dir.path().join("schema.graphql");
    fs::write(&schema_path, schema).unwrap();

    let addr = listen_address();

    let command = cmd!(
        cargo_bin("grafbase"),
        "federated",
        "start",
        "--listen-address",
        &addr.to_string(),
        "--config",
        &config_path.to_str().unwrap(),
        "--federated-schema",
        &schema_path.to_str().unwrap(),
    );

    let mut commands = CommandHandles::new();
    commands.push(command.start().unwrap());

    let endpoint = match path {
        Some(path) => format!("http://{addr}/{path}"),
        None => format!("http://{addr}/graphql"),
    };

    let mut client = Client::new(endpoint, format!("http://{addr}"), ClientOptions::default(), commands);

    if let Some(headers) = headers {
        for header in headers {
            client = client.with_header(header.0, header.1);
        }
    }

    let client = Arc::new(client);

    let res = catch_unwind(AssertUnwindSafe(|| {
        runtime().block_on(async {
            client.poll_endpoint(30, 300).await;
            test(client.clone()).await
        })
    }));

    client.kill_handles();

    res.unwrap();
}

fn with_hybrid_server<F, T>(config: &str, graph_ref: &str, sdl: &str, test: T)
where
    T: FnOnce(Arc<Client>) -> F,
    F: Future<Output = ()>,
{
    let temp_dir = tempdir().unwrap();

    let config_path = temp_dir.path().join("grafbase.toml");
    fs::write(&config_path, config).unwrap();

    let addr = listen_address();

    let uplink_response = serde_json::json!({
        "account_id": "01HR7NP3A4NDVWC10PZW6ZMC5P",
        "graph_id": "01HR7NPB8E3YW29S5PPSY1AQKR",
        "branch": "main",
        "branch_id": "01HR7NPB8E3YW29S5PPSY1AQKA",
        "sdl": sdl,
        "version_id": "01HR7NPYWWM6DEKACKKN3EPFP2",
    });

    let res = runtime().block_on(async {
        let response = ResponseTemplate::new(200).set_body_string(serde_json::to_string(&uplink_response).unwrap());
        let server = wiremock::MockServer::start().await;

        Mock::given(method("GET"))
            .and(path(format!("/graphs/{graph_ref}/current")))
            .and(header("Authorization", format!("Bearer {ACCESS_TOKEN}")))
            .respond_with(response)
            .mount(&server)
            .await;

        let command = cmd!(
            cargo_bin("grafbase"),
            "federated",
            "start",
            "--listen-address",
            &addr.to_string(),
            "--config",
            &config_path.to_str().unwrap(),
            "--graph-ref",
            graph_ref,
        )
        .env("GRAFBASE_GDN_URL", format!("http://{}", server.address()))
        .env("GRAFBASE_ACCESS_TOKEN", ACCESS_TOKEN);

        let mut commands = CommandHandles::new();
        commands.push(command.start().unwrap());

        let client = Arc::new(Client::new(
            format!("http://{addr}/graphql"),
            format!("http://{addr}"),
            ClientOptions::default(),
            commands,
        ));

        client.poll_endpoint(30, 300).await;

        let res = AssertUnwindSafe(test(client.clone())).catch_unwind().await;

        client.kill_handles();

        res
    });

    res.unwrap();
}

fn load_schema(name: &str) -> String {
    let path = format!("./tests/production_server/schemas/{name}.graphql");
    fs::read_to_string(path).unwrap()
}

pub fn introspect(url: &str) -> Output {
    let args = vec!["introspect", url];

    duct::cmd(cargo_bin("grafbase"), args)
        .stdout_capture()
        .stderr_capture()
        .unchecked()
        .run()
        .unwrap()
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
        let result: serde_json::Value = client.gql(query).send().await;
        let result = serde_json::to_string_pretty(&result).unwrap();

        insta::assert_snapshot!(&result, @r###"
        {
          "data": {},
          "errors": [
            {
              "message": "error sending request for url (http://127.0.0.1:46697/): client error (Connect)"
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
        let output = introspect(client.endpoint());
        let result = String::from_utf8_lossy(&output.stdout);

        insta::assert_snapshot!(&result, @r###"
        type Cart {
          products: [Product!]!
        }
        type Picture {
          height: Int!
          url: String!
          width: Int!
        }
        type Product {
          name: String!
          price: Int!
          reviews: [Review!]!
          upc: String!
        }
        type Query {
          me: User!
          topProducts: [Product!]!
        }
        type Review {
          author: User
          body: String!
          id: ID!
          pictures: [Picture!]!
          product: Product!
        }
        type Subscription {
          newProducts: Product!
        }
        type User {
          cart: Cart!
          id: ID!
          joinedTimestamp: Int!
          profilePicture: Picture
          reviewCount: Int!
          reviews: [Review!]!
          trustworthiness: Trustworthiness!
          username: String!
        }
        enum Trustworthiness {
          KINDA_TRUSTED
          NOT_TRUSTED
          REALLY_TRUSTED
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
        let output = introspect(client.endpoint());
        let result = String::from_utf8_lossy(&output.stdout);

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
        let result: serde_json::Value = client.gql(query).send().await;
        let result = serde_json::to_string_pretty(&result).unwrap();

        insta::assert_snapshot!(&result, @r###"
        {
          "data": {},
          "errors": [
            {
              "message": "error sending request for url (http://127.0.0.1:46697/): client error (Connect)"
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
        let response = client.gql::<serde_json::Value>(query).request().await;
        assert_eq!(http::StatusCode::FORBIDDEN, response.status());
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
        let result: serde_json::Value = client.gql(query).send().await;
        let result = serde_json::to_string_pretty(&result).unwrap();

        insta::assert_snapshot!(&result, @r###"
        {
          "data": {},
          "errors": [
            {
              "message": "error sending request for url (http://127.0.0.1:46697/): client error (Connect)"
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

    with_hybrid_server("", "test_graph", &schema, |client| async move {
        let result: serde_json::Value = client.gql(query).send().await;
        let result = serde_json::to_string_pretty(&result).unwrap();

        insta::assert_snapshot!(&result, @r###"
        {
          "data": {},
          "errors": [
            {
              "message": "error sending request for url (http://127.0.0.1:46697/): client error (Connect)"
            }
          ]
        }
        "###);
    });
}
