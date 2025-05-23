use graphql_mocks::Stateful;
use integration_tests::{gateway::Gateway, runtime};

#[test]
fn subgraph_retries_mutations_disabled() {
    runtime().block_on(async move {
        let config = indoc::indoc! {r#"
            [subgraphs.stateful.retry]
            enabled = true
            min_per_second = 1
            retry_percent = 0.01
        "#};

        let engine = Gateway::builder()
            .with_subgraph(Stateful::default())
            .with_toml_config(config)
            .build()
            .await;

        let response = engine.post("query { incrementAndFailIfLessThan(n: 3) }").await;

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "incrementAndFailIfLessThan": 3
          }
        }
        "###);

        // Now mutations: retries are not enabled for mutations.
        let response = engine.post("mutation { incrementAndFailIfLessThan(n: 5) }").await;

        insta::assert_json_snapshot!(response, {
            ".errors[0].message" => "REDACTED".to_owned(),
        }, @r#"
        {
          "data": null,
          "errors": [
            {
              "message": "REDACTED",
              "locations": [
                {
                  "line": 1,
                  "column": 12
                }
              ],
              "path": [
                "incrementAndFailIfLessThan"
              ],
              "extensions": {
                "code": "SUBGRAPH_REQUEST_ERROR"
              }
            }
          ]
        }
        "#);

        // Queries can still be retried...
        let response = engine.post("query { incrementAndFailIfLessThan(n: 7) }").await;

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "incrementAndFailIfLessThan": 7
          }
        }
        "###);

        // But not too often.
        let response = engine.post("query { incrementAndFailIfLessThan(n: 500) }").await;

        insta::assert_json_snapshot!(response, {
            ".errors[0].message" => "REDACTED".to_owned(),
        }, @r#"
        {
          "data": null,
          "errors": [
            {
              "message": "REDACTED",
              "locations": [
                {
                  "line": 1,
                  "column": 9
                }
              ],
              "path": [
                "incrementAndFailIfLessThan"
              ],
              "extensions": {
                "code": "SUBGRAPH_REQUEST_ERROR"
              }
            }
          ]
        }
        "#);
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

        let engine = Gateway::builder()
            .with_subgraph(Stateful::default())
            .with_toml_config(config)
            .build()
            .await;

        let response = engine.post("mutation { incrementAndFailIfLessThan(n: 3) }").await;

        insta::assert_json_snapshot!(response, {
            ".errors[0].message" => "REDACTED".to_owned(),
        }, @r#"
        {
          "data": {
            "incrementAndFailIfLessThan": 3
          }
        }
        "#);
    });
}
