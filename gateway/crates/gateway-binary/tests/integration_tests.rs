#![allow(unused_crate_dependencies)]

mod client;

use chrono::Utc;
use client::{load_schema, private_key, with_hybrid_server, with_static_server};
use duct::cmd;
use indoc::indoc;
use licensing::License;
use ulid::Ulid;

use crate::client::{cargo_bin, introspect, listen_address, with_config_files};

#[ctor::ctor]
fn setup_rustls() {
    rustls::crypto::ring::default_provider().install_default().unwrap();
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

    with_static_server("", &schema, None, None, None, |client| async move {
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
fn with_otel() {
    let config = indoc! {r#"
        [telemetry]
        service_name = "meow"

        [telemetry.tracing.exporters.stdout]
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

    with_static_server(config, &schema, None, None, None, |client| async move {
        let result: serde_json::Value = client.gql(query).send().await;
        serde_json::to_string_pretty(&result).unwrap();
    })
}

#[test]
fn introspect_enabled() {
    let config = indoc! {r#"
        [graph]
        introspection = true
    "#};

    let schema = load_schema("big");

    with_static_server(config, &schema, None, None, None, |client| async move {
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

    with_static_server(config, &schema, None, None, None, |client| async move {
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

    with_static_server(config, &schema, Some("custom"), None, None, |client| async move {
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

    with_static_server(config, &schema, Some("custom"), None, None, |client| async move {
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

    with_static_server(config, &schema, None, Some(headers), None, |client| async move {
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

#[test]
fn valid_license_and_enterprise_features() {
    let config = indoc! {r#"
        [operation_limits]
        depth = 3
        height = 10
        aliases = 100
        root_fields = 10
        complexity = 1000

        [subscriptions]
        enabled = true
    "#};

    let license = License {
        graph_id: Ulid::new(),
        account_id: Ulid::new(),
    };

    let schema = load_schema("big");
    let token = license.sign(private_key(), Utc::now()).unwrap();

    let query = indoc! {r#"
        query Me {
          me {
            id
          }
        }
    "#};

    with_static_server(config, &schema, None, None, Some(token.as_str()), |client| async move {
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

#[test]
fn no_license_with_enterprise_features() {
    let config = indoc! {r#"
        [operation_limits]
        depth = 3
        height = 10
        aliases = 100
        root_fields = 10
        complexity = 1000

        [subscriptions]
        enabled = true

        [trusted_documents]
        enabled = true

        [[authentication.providers]]

        [authentication.providers.jwt]
        name = "foo"

        [authentication.providers.jwt.jwks]
        url = "https://example.com/.well-known/jwks.json"
        issuer = "https://example.com/"
        audience = "my-project"
        poll_interval = "60s"
    "#};

    let schema = load_schema("big");
    let project = with_config_files(config, Some(&schema), None);
    let addr = listen_address();

    let command = cmd!(
        cargo_bin("grafbase-gateway"),
        "--listen-address",
        &addr.to_string(),
        "--config",
        &project.config_path.to_str().unwrap(),
        "--schema",
        &project.schema_path.unwrap().to_str().unwrap(),
    )
    .unchecked()
    .stdout_capture()
    .stderr_capture();

    let handle = command.start().unwrap();
    let error = handle.into_output().unwrap();
    let stderr = String::from_utf8_lossy(&error.stderr);

    insta::assert_snapshot!(&stderr, @r###"
    Error: the following features are only available with a valid license: operation limits, trusted documents, authentication, subscriptions
    "###);
}
