use integration_tests::{gateway::Gateway, runtime};
use serde_json::json;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

#[test]
fn should_not_fail_because_of_status_code_if_response_is_correct() {
    runtime().block_on(async move {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(400).set_body_json(json!({
                "data": {
                    "id": "abc"
                }
            })))
            .mount(&mock_server)
            .await;

        let url = mock_server.uri();

        let engine = Gateway::builder()
            .with_federated_sdl(format!(
                r#"
                type Query
                    @join__type(graph: A)
                {{
                    id: ID!
                }}

                enum join__Graph
                {{
                    A @join__graph(name: "a", url: "{url}")
                }}
            "#
            ))
            .build()
            .await;

        let response = engine.post(r#"query { id }"#).await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "data": {
            "id": "abc"
          }
        }
        "#);
    })
}
