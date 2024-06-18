#![allow(unused_crate_dependencies, clippy::panic)]
mod utils;

use backend::project::GraphType;
use serde_json::Value;
use utils::environment::Environment;

const SCHEMA: &str = r#"
type Todo {
    id: ID!
    title: String
}

extend type Query {
    todo(id: ID!): Todo @resolver(name: "todo")
}
"#;

#[ctor::ctor]
fn setup_rustls() {
    rustls::crypto::ring::default_provider().install_default().unwrap();
}

#[rstest::rstest]
#[case::get(reqwest::Method::GET)]
#[case::post(reqwest::Method::POST)]
#[tokio::test(flavor = "multi_thread")]
async fn automatic_persisted_queries(#[case] method: reqwest::Method) {
    let mut env = Environment::init_async().await;
    env.grafbase_init(GraphType::Standalone);
    env.write_schema(SCHEMA);
    env.grafbase_dev_watch();
    env.write_file(
        "resolvers/todo.js",
        r#"
        export default function Resolver(_, {id}) {
            if (id === "1") {
                return {id: "1", title: "title"};
            } else {
                return null;
            }
        }
        "#,
    );

    let client = env.create_async_client().with_api_key();

    client.poll_endpoint(30, 300).await;

    let query = r#"
        query {
            todo(id: "1") {
                id
                title
            }
        }
    "#;

    let execute = |query: &'static str, extensions: &Value| {
        if method == reqwest::Method::GET {
            client.gql_get::<Value>(query).extensions(extensions)
        } else {
            client.gql::<Value>(query).extensions(extensions)
        }
    };

    let apq_ext = serde_json::json!({
        "persistedQuery": {
            "version": 1,
            "sha256Hash": sha256(query)
        }
    });

    // Missing query
    insta::assert_json_snapshot!(execute("", &apq_ext).await, @r###"
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
    insta::assert_json_snapshot!(execute(query, &apq_ext).await, @r###"
    {
      "data": {
        "todo": {
          "id": "1",
          "title": "title"
        }
      }
    }
    "###);

    // Query isn't necessary anymore
    insta::assert_json_snapshot!(execute("", &apq_ext).await, @r###"
    {
      "data": {
        "todo": {
          "id": "1",
          "title": "title"
        }
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
    insta::assert_json_snapshot!(execute(query, &invalid_version).await, @r###"
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
    insta::assert_json_snapshot!(execute(query, &invalid_version).await, @r###"
    {
      "errors": [
        {
          "message": "Persisted query version not supported"
        }
      ]
    }
    "###);
}

fn sha256(query: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest = <Sha256 as Digest>::digest(query.as_bytes());
    hex::encode(digest)
}
