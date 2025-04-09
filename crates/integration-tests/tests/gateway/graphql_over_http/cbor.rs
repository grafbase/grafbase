use graphql_mocks::FakeGithubSchema;
use integration_tests::{gateway::Gateway, runtime};

#[test]
fn content_type() {
    runtime().block_on(async move {
        let engine = Gateway::builder().with_subgraph(FakeGithubSchema).build().await;

        let response = engine
            .raw_execute(
                http::Request::builder()
                    .uri("http://localhost/graphql")
                    .method(http::Method::POST)
                    .header(http::header::ACCEPT, "application/json")
                    .header(http::header::CONTENT_TYPE, "application/cbor")
                    .body(minicbor_serde::to_vec(serde_json::json!({"query": "{ __typename }"})).unwrap())
                    .unwrap(),
            )
            .await;
        let status = response.status();
        let body: serde_json::Value = serde_json::from_slice(&response.into_body()).unwrap();
        insta::assert_json_snapshot!(body, @r#"
        {
          "data": {
            "__typename": "Query"
          }
        }
        "#);
        assert_eq!(status, 200);
    })
}
