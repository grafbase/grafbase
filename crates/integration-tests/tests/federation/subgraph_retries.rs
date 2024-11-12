use engine_v2::Engine;
use graphql_mocks::Stateful;
use integration_tests::{federation::EngineV2Ext, runtime};

#[test]
fn subgraph_retries_mutations_disabled() {
    runtime().block_on(async move {
        let config = indoc::indoc! {r#"
            [subgraphs.stateful.retry]
            enabled = true
            min_per_second = 1
            retry_percent = 0.01
        "#};

        let engine = Engine::builder()
            .with_subgraph(Stateful::default())
            .with_toml_config(config)
            .build()
            .await;

        let response = engine.post("query { incrementAndFailIfLessThan(n: 5) }").await;

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "incrementAndFailIfLessThan": 5
          }
        }
        "###);

        // Now mutations: retries are not enabled for mutations.
        let response = engine.post("mutation { incrementAndFailIfLessThan(n: 7) }").await;

        insta::assert_json_snapshot!(response, {
            ".errors[0].message" => "REDACTED".to_owned(),
        });

        // Queries can still be retried...
        let response = engine.post("query { incrementAndFailIfLessThan(n: 10) }").await;

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "incrementAndFailIfLessThan": 10
          }
        }
        "###);

        // But not too often.
        let response = engine.post("query { incrementAndFailIfLessThan(n: 500) }").await;

        insta::assert_json_snapshot!(response, {
            ".errors[0].message" => "REDACTED".to_owned(),
        });
    });
}

#[test]
fn subgraph_retries_mutations_enabled() {
    runtime().block_on(async move {
        let config = indoc::indoc! {r#"
            [subgraphs.stateful.retry]
            enabled = true
            retry_mutations = true
        "#};

        let engine = Engine::builder()
            .with_subgraph(Stateful::default())
            .with_toml_config(config)
            .build()
            .await;

        let response = engine.post("mutation { incrementAndFailIfLessThan(n: 5) }").await;

        insta::assert_json_snapshot!(response, {
            ".errors[0].message" => "REDACTED".to_owned(),
        });
    });
}
