use std::time::Duration;

use grafbase_docker_tests::{retry_for, with_grafbase};

#[test]
fn hello_world_resolver() {
    let tmp_dir = tempfile::tempdir().unwrap();
    std::fs::write(
        tmp_dir.path().join("grafbase.config.ts"),
        r###"
        import { g, config } from '@grafbase/sdk'

        g.query('hello', {
          args: { name: g.string().optional() },
          returns: g.string(),
          resolver: 'hello',
        })

        export default config({
          graph: g,
          auth: {
            rules: (rules) => {
              rules.public()
            }
          }
        })
        "###,
    )
    .unwrap();
    std::fs::create_dir(tmp_dir.path().join("resolvers")).unwrap();
    std::fs::write(
        tmp_dir.path().join("resolvers").join("hello.js"),
        r###"
        export default function Resolver(_, { name }) {
          return `Hello ${name || 'world'}!`
        }
        "###,
    )
    .unwrap();

    with_grafbase(tmp_dir, |url| async move {
        // quick hack to let grafbase start.
        tokio::time::sleep(Duration::from_secs(2)).await;

        let client = reqwest::Client::new();
        let response = retry_for(10, Duration::from_secs(1), || async {
            client
                .post(&url)
                .json(&serde_json::json!({
                    "query": "{ hello }"
                }))
                .send()
                .await
                .map_err(Into::into)
        })
        .await
        .unwrap();
        assert_eq!(response.status(), 200);
        let body: serde_json::Value = response.json().await.unwrap();
        insta::assert_json_snapshot!(body, @r###"
        {
          "data": {
            "hello": "Hello world!"
          }
        }
        "###);
    })
}
