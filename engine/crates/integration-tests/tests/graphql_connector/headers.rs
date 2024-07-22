use graphql_mocks::{EchoSchema, MockGraphQlServer};
use integration_tests::{runtime, EngineBuilder, ResponseExt};

#[test]
fn test_header_forwarding() {
    runtime().block_on(async {
        let graphql_mock = MockGraphQlServer::new(EchoSchema).await;

        let engine = EngineBuilder::new(schema(graphql_mock.port()))
            .with_env_var("API_KEY", "BLAH")
            .build()
            .await;

        let response = engine
            .execute(
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
            .await
            .into_value();

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "headers": [
              {
                "name": "accept",
                "value": "*/*"
              },
              {
                "name": "another-one",
                "value": "yes"
              },
              {
                "name": "authorization",
                "value": "Bearer BLAH"
              },
              {
                "name": "content-length",
                "value": "96"
              },
              {
                "name": "content-type",
                "value": "application/json"
              },
              {
                "name": "user-agent",
                "value": "Grafbase"
              },
              {
                "name": "wow-what-a-header",
                "value": "isn't it the best"
              }
            ]
          }
        }
        "###);
    });
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
