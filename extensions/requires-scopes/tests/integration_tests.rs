mod hydra;

use grafbase_sdk::test::{DynamicSchema, TestConfig, TestRunner};
use hydra::{ADMIN_SCOPE, CoreClientExt as _, JWKS_URI, OryHydraOpenIDProvider, READ_SCOPE, WRITE_SCOPE};

const CLI_PATH: &str = "../../target/debug/grafbase";
const GATEWAY_PATH: &str = "../../target/debug/grafbase-gateway";

async fn setup(scopes: Option<&str>) -> (TestRunner, String) {
    let extension_path = std::env::current_dir().unwrap().join("build");
    let path_str = format!("file://{}", extension_path.display());

    // Create a subgraph with a single field
    let subgraph = DynamicSchema::builder(format!(
        r#"
        extend schema
            @link(url: "{path_str}", import: ["@requiresScopes"])

        type Query {{
            public: String
            hasReadScope: String @requiresScopes(scopes: "read")
            hasReadAndWriteScope: String @requiresScopes(scopes: [["read", "write"]])
            hasReadOrWriteScope: String @requiresScopes(scopes: [["read"], ["write"]])
        }}
        "#
    ))
    .with_resolver("Query", "public", String::from("public"))
    .with_resolver("Query", "hasReadScope", String::from("Has read scope"))
    .with_resolver(
        "Query",
        "hasReadAndWriteScope",
        String::from("Has read and write scope"),
    )
    .with_resolver("Query", "hasReadOrWriteScope", String::from("Has read or write scope"))
    .into_subgraph("test")
    .unwrap();

    let config = format!(
        r#"
        [[authentication.providers]]

        [authentication.providers.jwt]
        name = "my-jwt"

        [authentication.providers.jwt.jwks]
        url = "{JWKS_URI}"

        [[authentication.providers]]

        [authentication.providers.anonymous]
        "#
    );

    // The test configuration is built with the subgraph and networking enabled.
    // You must have the CLI and Grafbase Gateway for this to work. If you do not have
    // them in the PATH, you can specify the paths to the executables with the `.with_cli` and
    // `.with_gateway` methods.
    let config = TestConfig::builder()
        .with_cli(CLI_PATH)
        .with_gateway(GATEWAY_PATH)
        .with_subgraph(subgraph)
        .enable_stdout()
        .enable_stderr()
        .build(config)
        .unwrap();

    // A runner for building the extension, and executing the Grafbase Gateway together
    // with the subgraphs. The runner composes all subgraphs into a federated schema.
    let runner = TestRunner::new(config).await.unwrap();

    let extra_params = if let Some(scopes) = scopes {
        vec![("scope", scopes)]
    } else {
        vec![]
    };
    let token = OryHydraOpenIDProvider::default()
        .create_client()
        .await
        .get_access_token_with_client_credentials(&extra_params)
        .await;

    (runner, token)
}

const QUERY: &str = r#"
query {
    public
    hasReadScope
    hasReadAndWriteScope
    hasReadOrWriteScope
}
"#;

#[tokio::test]
async fn anonymous_token() {
    let (runner, _) = setup(None).await;

    let result: serde_json::Value = runner.graphql_query(QUERY).send().await.unwrap();

    // The result is compared against a snapshot.
    insta::assert_json_snapshot!(result, @r#"
    {
      "data": {
        "public": "public",
        "hasReadScope": null,
        "hasReadAndWriteScope": null,
        "hasReadOrWriteScope": null
      },
      "errors": [
        {
          "message": "Not authorized",
          "locations": [
            {
              "line": 4,
              "column": 5
            }
          ],
          "path": [
            "hasReadScope"
          ],
          "extensions": {
            "code": "UNAUTHORIZED"
          }
        },
        {
          "message": "Not authorized",
          "locations": [
            {
              "line": 5,
              "column": 5
            }
          ],
          "path": [
            "hasReadAndWriteScope"
          ],
          "extensions": {
            "code": "UNAUTHORIZED"
          }
        },
        {
          "message": "Not authorized",
          "locations": [
            {
              "line": 6,
              "column": 5
            }
          ],
          "path": [
            "hasReadOrWriteScope"
          ],
          "extensions": {
            "code": "UNAUTHORIZED"
          }
        }
      ]
    }
    "#);
}

#[tokio::test]
async fn token_without_scopes() {
    let (runner, token) = setup(None).await;

    let result: serde_json::Value = runner
        .graphql_query(QUERY)
        .with_header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    // The result is compared against a snapshot.
    insta::assert_json_snapshot!(result, @r#"
    {
      "data": {
        "public": "public",
        "hasReadScope": null,
        "hasReadAndWriteScope": null,
        "hasReadOrWriteScope": null
      },
      "errors": [
        {
          "message": "Not authorized: insufficient scopes",
          "locations": [
            {
              "line": 4,
              "column": 5
            }
          ],
          "path": [
            "hasReadScope"
          ],
          "extensions": {
            "code": "UNAUTHORIZED"
          }
        },
        {
          "message": "Not authorized: insufficient scopes",
          "locations": [
            {
              "line": 5,
              "column": 5
            }
          ],
          "path": [
            "hasReadAndWriteScope"
          ],
          "extensions": {
            "code": "UNAUTHORIZED"
          }
        },
        {
          "message": "Not authorized: insufficient scopes",
          "locations": [
            {
              "line": 6,
              "column": 5
            }
          ],
          "path": [
            "hasReadOrWriteScope"
          ],
          "extensions": {
            "code": "UNAUTHORIZED"
          }
        }
      ]
    }
    "#);
}

#[tokio::test]
async fn token_with_insufficient_scopes() {
    let (runner, token) = setup(Some(ADMIN_SCOPE)).await;

    let result: serde_json::Value = runner
        .graphql_query(QUERY)
        .with_header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    // The result is compared against a snapshot.
    insta::assert_json_snapshot!(result, @r#"
    {
      "data": {
        "public": "public",
        "hasReadScope": null,
        "hasReadAndWriteScope": null,
        "hasReadOrWriteScope": null
      },
      "errors": [
        {
          "message": "Not authorized: insufficient scopes",
          "locations": [
            {
              "line": 4,
              "column": 5
            }
          ],
          "path": [
            "hasReadScope"
          ],
          "extensions": {
            "code": "UNAUTHORIZED"
          }
        },
        {
          "message": "Not authorized: insufficient scopes",
          "locations": [
            {
              "line": 5,
              "column": 5
            }
          ],
          "path": [
            "hasReadAndWriteScope"
          ],
          "extensions": {
            "code": "UNAUTHORIZED"
          }
        },
        {
          "message": "Not authorized: insufficient scopes",
          "locations": [
            {
              "line": 6,
              "column": 5
            }
          ],
          "path": [
            "hasReadOrWriteScope"
          ],
          "extensions": {
            "code": "UNAUTHORIZED"
          }
        }
      ]
    }
    "#);
}

#[tokio::test]
async fn token_with_read_scope() {
    let (runner, token) = setup(Some(READ_SCOPE)).await;

    let result: serde_json::Value = runner
        .graphql_query(QUERY)
        .with_header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    // The result is compared against a snapshot.
    insta::assert_json_snapshot!(result, @r#"
    {
      "data": {
        "public": "public",
        "hasReadScope": "Has read scope",
        "hasReadAndWriteScope": null,
        "hasReadOrWriteScope": "Has read or write scope"
      },
      "errors": [
        {
          "message": "Not authorized: insufficient scopes",
          "locations": [
            {
              "line": 5,
              "column": 5
            }
          ],
          "path": [
            "hasReadAndWriteScope"
          ],
          "extensions": {
            "code": "UNAUTHORIZED"
          }
        }
      ]
    }
    "#);
}

#[tokio::test]
async fn token_with_write_scope() {
    let (runner, token) = setup(Some(WRITE_SCOPE)).await;

    let result: serde_json::Value = runner
        .graphql_query(QUERY)
        .with_header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    // The result is compared against a snapshot.
    insta::assert_json_snapshot!(result, @r#"
    {
      "data": {
        "public": "public",
        "hasReadScope": null,
        "hasReadAndWriteScope": null,
        "hasReadOrWriteScope": "Has read or write scope"
      },
      "errors": [
        {
          "message": "Not authorized: insufficient scopes",
          "locations": [
            {
              "line": 4,
              "column": 5
            }
          ],
          "path": [
            "hasReadScope"
          ],
          "extensions": {
            "code": "UNAUTHORIZED"
          }
        },
        {
          "message": "Not authorized: insufficient scopes",
          "locations": [
            {
              "line": 5,
              "column": 5
            }
          ],
          "path": [
            "hasReadAndWriteScope"
          ],
          "extensions": {
            "code": "UNAUTHORIZED"
          }
        }
      ]
    }
    "#);
}

#[tokio::test]
async fn token_with_read_and_write_scopes() {
    let (runner, token) = setup(Some(&format!("{READ_SCOPE} {WRITE_SCOPE} {ADMIN_SCOPE}"))).await;

    let result: serde_json::Value = runner
        .graphql_query(QUERY)
        .with_header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    // The result is compared against a snapshot.
    insta::assert_json_snapshot!(result, @r#"
    {
      "data": {
        "public": "public",
        "hasReadScope": "Has read scope",
        "hasReadAndWriteScope": "Has read and write scope",
        "hasReadOrWriteScope": "Has read or write scope"
      }
    }
    "#);
}
