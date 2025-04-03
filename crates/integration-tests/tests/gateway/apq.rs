use graphql_mocks::FakeGithubSchema;
use indoc::indoc;
use integration_tests::{gateway::Gateway, runtime};

#[test]
fn when_disabled() {
    runtime().block_on(async move {
        let config = indoc! {r#"
            [apq]
            enabled = false
        "#};

        let engine = Gateway::builder()
            .with_toml_config(config)
            .with_subgraph(FakeGithubSchema)
            .build()
            .await;

        let query = "query { serverVersion }";
        let execute = |query: &'static str, extensions: &serde_json::Value| engine.post(query).extensions(extensions);

        let apq_ext = serde_json::json!({
            "persistedQuery": {
                "version": 1,
                "sha256Hash": sha256(query)
            }
        });

        insta::assert_json_snapshot!(execute(query, &apq_ext).await, @r#"
        {
          "errors": [
            {
              "message": "Persisted query not found",
              "extensions": {
                "code": "PERSISTED_QUERY_NOT_FOUND"
              }
            }
          ]
        }
        "#);

        insta::assert_json_snapshot!(execute("", &apq_ext).await, @r#"
        {
          "errors": [
            {
              "message": "Persisted query not found",
              "extensions": {
                "code": "PERSISTED_QUERY_NOT_FOUND"
              }
            }
          ]
        }
        "#);
    })
}

#[test]
fn single_field_from_single_server() {
    runtime().block_on(async move {
        let engine = Gateway::builder().with_subgraph(FakeGithubSchema).build().await;

        let query = "query { serverVersion }";
        let apq_ext = serde_json::json!({
            "persistedQuery": {
                "version": 1,
                "sha256Hash": sha256(query)
            }
        });

        let execute =
            |query: Option<&'static str>, extensions: &serde_json::Value| engine.post(query).extensions(extensions);

        // Missing query
        insta::assert_json_snapshot!(execute(Some(""), &apq_ext).await, @r###"
        {
          "errors": [
            {
              "message": "Persisted query not found",
              "extensions": {
                "code": "PERSISTED_QUERY_NOT_FOUND"
              }
            }
          ]
        }
        "###);

        insta::assert_json_snapshot!(execute(None, &apq_ext).await, @r###"
        {
          "errors": [
            {
              "message": "Persisted query not found",
              "extensions": {
                "code": "PERSISTED_QUERY_NOT_FOUND"
              }
            }
          ]
        }
        "###);

        // Providing the query
        insta::assert_json_snapshot!(execute(Some(query), &apq_ext).await, @r###"
        {
          "data": {
            "serverVersion": "1"
          }
        }
        "###);

        // Query isn't necessary anymore
        insta::assert_json_snapshot!(execute(None, &apq_ext).await, @r###"
        {
          "data": {
            "serverVersion": "1"
          }
        }
        "###);
        insta::assert_json_snapshot!(execute(Some(""), &apq_ext).await, @r###"
        {
          "data": {
            "serverVersion": "1"
          }
        }
        "###);

        // Wrong hash
        let invalid_hash = serde_json::json!({
            "persistedQuery": {
                "version": 1,
                "sha256Hash": sha256("query { todo { id title } }")
            }
        });
        insta::assert_json_snapshot!(execute(Some(query), &invalid_hash).await, @r###"
        {
          "errors": [
            {
              "message": "Invalid persisted query sha256Hash",
              "extensions": {
                "code": "PERSISTED_QUERY_ERROR"
              }
            }
          ]
        }
        "###);

        // Wrong version
        let invalid_version = serde_json::json!({
            "persistedQuery": {
                "version": 2,
                "sha256Hash": sha256(query)
            }
        });
        insta::assert_json_snapshot!(execute(Some(query), &invalid_version).await, @r###"
        {
          "errors": [
            {
              "message": "Persisted query version not supported",
              "extensions": {
                "code": "PERSISTED_QUERY_ERROR"
              }
            }
          ]
        }
        "###);
    });
}

fn sha256(query: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest = <Sha256 as Digest>::digest(query.as_bytes());
    hex::encode(digest)
}
