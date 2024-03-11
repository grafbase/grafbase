use engine_v2::Engine;
use graphql_mocks::{FakeGithubSchema, MockGraphQlServer};
use integration_tests::{engine::GraphQlRequest, federation::EngineV2Ext, runtime};

#[test]
fn single_field_from_single_server() {
    runtime().block_on(async move {
        let github_mock = MockGraphQlServer::new(FakeGithubSchema).await;

        let engine = Engine::builder()
            .with_schema("schema", &github_mock)
            .await
            .finish()
            .await;

        let query = "query { serverVersion }";

        let execute = |query: Option<&'static str>, extensions: &serde_json::Value| {
            engine
                .execute(query.map(GraphQlRequest::from).unwrap_or_default())
                .extensions(extensions)
        };

        let apq_ext = serde_json::json!({
            "persistedQuery": {
                "version": 1,
                "sha256Hash": sha256(query)
            }
        });

        // Missing query
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

        // Wrong hash
        let invalid_version = serde_json::json!({
            "persistedQuery": {
                "version": 1,
                "sha256Hash": sha256("query { todo { id title } }")
            }
        });
        insta::assert_json_snapshot!(execute(Some(query), &invalid_version).await, @r###"
        {
          "errors": [
            {
              "message": "Invalid persisted query sha256Hash"
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
              "message": "Persisted query version not supported"
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
