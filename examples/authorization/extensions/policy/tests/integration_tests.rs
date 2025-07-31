use grafbase_sdk::test::{GraphqlSubgraph, TestGateway};
use serde_json::json;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

async fn mock_server() -> MockServer {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/policy"))
        .respond_with(|request: &wiremock::Request| {
            #[derive(serde::Deserialize)]
            struct Request {
                policies: Vec<String>,
            }

            #[derive(serde::Serialize)]
            struct Response {
                granted: Vec<bool>,
            }

            let Request { policies } = request.body_json::<Request>().unwrap();
            let response = Response {
                granted: policies.iter().map(|policy| policy.starts_with("read")).collect(),
            };

            ResponseTemplate::new(200).set_body_json(response)
        })
        .mount(&mock_server)
        .await;

    mock_server
}

#[tokio::test]
async fn test() {
    let mock = mock_server().await;
    let gateway = TestGateway::builder()
        .subgraph(
            GraphqlSubgraph::with_schema(
                r#"
                extend schema @link(url: "<self>", import: ["@policy"])

                type Query {
                    check: Check
                }

                type Check {
                    read: String @policy(policies: [["read"]])
                    write: String @policy(policies: [["write"]])
                    read_or_write: String @policy(policies: [["read"], ["write"]])
                    read_and_write: String @policy(policies: [["read", "write"]])
                }
                "#,
            )
            .with_resolver(
                "Query",
                "check",
                json!({
                    "read": "R",
                    "write": "W",
                    "read_or_write": "R || W",
                    "read_and_write": "R && W",
                }),
            ),
        )
        .toml_config(format!(
            r#"
            [extensions.policy.config]
            auth_service_url = "{}"
            "#,
            mock.uri()
        ))
        .stream_stdout_stderr()
        .build()
        .await
        .unwrap();

    let response = gateway
        .query(r#"query { check { read write read_or_write read_and_write } }"#)
        .send()
        .await;

    // The result is compared against a snapshot.
    insta::assert_json_snapshot!(response, @r#"
    {
      "data": {
        "check": {
          "read": "R",
          "write": null,
          "read_or_write": "R || W",
          "read_and_write": null
        }
      },
      "errors": [
        {
          "message": "Not authorized: policy not granted.",
          "locations": [
            {
              "line": 1,
              "column": 22
            }
          ],
          "path": [
            "check",
            "write"
          ],
          "extensions": {
            "code": "UNAUTHORIZED"
          }
        }
      ]
    }
    "#);
}
