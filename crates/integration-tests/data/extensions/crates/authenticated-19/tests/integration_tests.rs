mod hydra;

use grafbase_sdk::test::{GraphqlSubgraph, TestGateway};
use hydra::{CoreClientExt as _, JWKS_URI, OryHydraOpenIDProvider};
use indoc::formatdoc;

#[tokio::test]
async fn test_authenticated() {
    let gateway = TestGateway::builder()
        .toml_config(formatdoc!(
            r#"
            [graph]
            introspection = true

            [authentication]
            default = "anonymous"

            [extensions.jwt]
            version = "1.3"

            [extensions.jwt.config]
            url = "{JWKS_URI}"
            "#,
        ))
        .subgraph(
            GraphqlSubgraph::with_schema(
                r#"
            extend schema
                @link(url: "<self>", import: ["@authenticated"])

            type Query {
                public: String
                private: String @authenticated
            }
            "#,
            )
            .with_resolver("Query", "public", "public")
            .with_resolver("Query", "private", "private"),
        )
        .build()
        .await
        .unwrap();

    let token = OryHydraOpenIDProvider::default()
        .create_client()
        .await
        .get_access_token_with_client_credentials(&[])
        .await;

    println!("{}", gateway.introspect().send().await);

    let response = gateway.query(r#"query { public private }"#).send().await;

    insta::assert_json_snapshot!(response, @r#"
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

    let response = gateway
        .query(r#"query { public private }"#)
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await;

    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "public": "public",
        "private": "private"
      }
    }
    "#);
}
