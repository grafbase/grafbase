use integration_tests::{federation::Gateway, runtime};

#[test]
fn works_with_empty_config() {
    runtime().block_on(async {
        let engine = Gateway::builder().build().await;
        let response = engine.post("{ __typename }").await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "__typename": "Query"
          }
        }
        "###);
    });
}
