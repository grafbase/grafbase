use engine::Engine;
use integration_tests::{federation::EngineExt, runtime};

#[test]
fn works_with_empty_config() {
    runtime().block_on(async {
        let engine = Engine::builder().build().await;
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
