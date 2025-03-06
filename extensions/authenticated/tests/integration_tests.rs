mod hydra;

use grafbase_sdk::test::{DynamicSchema, TestConfig, TestRunner};
use hydra::{CoreClientExt as _, JWKS_URI, OryHydraOpenIDProvider};

const CLI_PATH: &str = "../../target/debug/grafbase";
const GATEWAY_PATH: &str = "../../target/debug/grafbase-gateway";

#[tokio::test]
async fn test_authenticated() {
    let extension_path = std::env::current_dir().unwrap().join("build");
    let path_str = format!("file://{}", extension_path.display());

    // Create a subgraph with a single field
    let subgraph = DynamicSchema::builder(format!(
        r#"
        extend schema
            @link(url: "{path_str}", import: ["@authenticated"])

        type Query {{
            public: String
            private: String @authenticated
        }}
        "#
    ))
    .with_resolver("Query", "public", String::from("public"))
    .with_resolver("Query", "private", String::from("private"))
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

    let token = OryHydraOpenIDProvider::default()
        .create_client()
        .await
        .get_access_token_with_client_credentials(&[])
        .await;

    let result: serde_json::Value = runner
        .graphql_query(r#"query { public private }"#)
        .send()
        .await
        .unwrap();

    // The result is compared against a snapshot.
    insta::assert_json_snapshot!(result, @r#"
    {
      "data": {
        "public": "public",
        "private": null
      },
      "errors": [
        {
          "message": "Not authenticated",
          "locations": [
            {
              "line": 1,
              "column": 16
            }
          ],
          "path": [
            "private"
          ],
          "extensions": {
            "code": "UNAUTHORIZED"
          }
        }
      ]
    }
    "#);

    let result: serde_json::Value = runner
        .graphql_query(r#"query { public private }"#)
        .with_header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    // The result is compared against a snapshot.
    insta::assert_json_snapshot!(result, @r#"
    {
      "data": {
        "public": "public",
        "private": "private"
      }
    }
    "#);
}
