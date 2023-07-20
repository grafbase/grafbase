use backend::project::ConfigType;
use serde_json::Value;

use crate::{
    server,
    utils::{async_client::AsyncClient, environment::Environment},
};

#[tokio::test(flavor = "multi_thread")]
async fn test_header_forwarding() {
    server::run(54303).await;

    let mut env = Environment::init_async().await;
    let client = start_grafbase(&mut env, schema(54303)).await;

    let response = client
        .gql::<Value>(
            r#"
                query {
                    headers {
                        name
                        value
                    }
                }
        "#,
        )
        .header("wow-what-a-header", "isn't it the best")
        .header("and-another-one", "yes")
        .header("a-header-that-shouldnt-be-forwarded", "ok")
        .header("Authorization", "Basic XYZ")
        .await;

    insta::assert_yaml_snapshot!(response, @r###"
    ---
    data:
      headers:
        - name: host
          value: "127.0.0.1:54303"
        - name: connection
          value: keep-alive
        - name: another-one
          value: "yes"
        - name: authorization
          value: Bearer BLAH
        - name: content-type
          value: application/json
        - name: user-agent
          value: Grafbase
        - name: wow-what-a-header
          value: "isn't it the best"
        - name: mf-loop
          value: "1"
        - name: accept-encoding
          value: "gzip, deflate"
        - name: content-length
          value: "96"
    "###);
}

async fn start_grafbase(env: &mut Environment, schema: impl AsRef<str>) -> AsyncClient {
    env.grafbase_init(ConfigType::GraphQL);
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
