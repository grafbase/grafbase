use std::fmt::Display;

use backend::project::GraphType;
use serde_json::{json, Value};

use crate::{
    server,
    utils::{async_client::AsyncClient, environment::Environment},
};

#[tokio::test(flavor = "multi_thread")]
async fn test_header_forwarding() {
    let port = server::run().await;

    let mut env = Environment::init_async().await;
    let client = start_grafbase(&mut env, schema(port)).await;

    let mut response = client
        .gql::<Value>(
            r"
                query {
                    headers {
                        name
                        value
                    }
                }
        ",
        )
        .header("wow-what-a-header", "isn't it the best")
        .header("and-another-one", "yes")
        .header("a-header-that-shouldnt-be-forwarded", "ok")
        .header("Authorization", "Basic XYZ")
        .await;

    // Remove the host header because it's dynamic
    response.get_mut("data").and_then(|data| {
        let headers = data.get_mut("headers")?;

        let host_header_index = headers
            .as_array()?
            .iter()
            .enumerate()
            .find(|(_, header)| header.get("name") == Some(&json!("host")))?
            .0;

        headers.as_array_mut()?.remove(host_header_index);
        Some(())
    });

    insta::assert_yaml_snapshot!(response, @r###"
    ---
    data:
      headers:
        - name: user-agent
          value: Grafbase
        - name: content-type
          value: application/json
        - name: authorization
          value: Bearer BLAH
        - name: wow-what-a-header
          value: "isn't it the best"
        - name: another-one
          value: "yes"
        - name: accept
          value: "*/*"
        - name: content-length
          value: "96"
    "###);
}

async fn start_grafbase(env: &mut Environment, schema: impl AsRef<str> + Display) -> AsyncClient {
    env.grafbase_init(GraphType::Single);
    env.write_schema(schema);
    env.set_variables([("API_KEY", "BLAH")]);
    env.grafbase_dev_watch();

    let client = env.create_async_client().with_api_key();

    client.poll_endpoint(30, 300).await;

    client
}

fn schema(port: u16) -> String {
    format!(
        r#"
          extend schema
          @graphql(
            name: "Test",
            namespace: false,
            url: "http://127.0.0.1:{port}",
            schema: "http://127.0.0.1:{port}/spec.json",
            headers: [
                {{ name: "authorization", value: "Bearer {{{{ env.API_KEY }}}}" }}
                {{ name: "Wow-what-a-header", forward: "Wow-what-a-header" }}
                {{ name: "another-one", forward: "and-another-one" }}
                {{ name: "secret-third-header", forward: "secret-third-header" }}
            ],
          )
        "#
    )
}
