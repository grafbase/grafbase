//! Tests of the execute_stream functionality in engine-v2
//!
//! Subscrition specific tests will probably live elsewhere

use engine_v2::Engine;
use graphql_mocks::Stateful;
use integration_tests::{federation::EngineV2Ext, runtime};

#[test]
fn can_run_a_query_via_execute_stream() {
    runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(Stateful::default()).build().await;

        let response = engine.post("query { value }").into_multipart_stream().await;

        insta::assert_json_snapshot!(response.collected_body, @r###"
        [
          {
            "data": {
              "value": 0
            }
          }
        ]
        "###);
    })
}

#[test]
fn can_run_a_mutation_via_execute_stream() {
    runtime().block_on(async move {
        let engine = Engine::builder().with_subgraph(Stateful::default()).build().await;

        let response = engine
            .post(
                r"
                mutation {
                    first: set(val: 1)
                    second: multiply(by: 2)
                    third: multiply(by: 7)
                    fourth: set(val: 3)
                    fifth: multiply(by: 11)
                }
                ",
            )
            .into_multipart_stream()
            .await;

        insta::assert_json_snapshot!(response.collected_body, @r###"
        [
          {
            "data": {
              "first": 1,
              "second": 2,
              "third": 14,
              "fourth": 3,
              "fifth": 33
            }
          }
        ]
        "###);
    })
}
