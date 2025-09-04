use futures::StreamExt;
use futures::stream::FuturesUnordered;
use graphql_mocks::SlowSchema;
use integration_tests::{gateway::Gateway, runtime};

#[test]
fn inflight_deduplication_enabled() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph(SlowSchema::default())
            .with_toml_config(
                r###"
                [traffic_shaping]
                inflight_deduplication = true
                "###,
            )
            .build()
            .await;

        let responses = (0..10)
            .map(|_| async { gateway.post("query { slow: delay(ms: 100) }").await })
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>()
            .await;

        insta::assert_json_snapshot!(responses[0], @r#"
        {
          "data": {
            "slow": 100
          }
        }
        "#);

        insta::assert_json_snapshot!(gateway.drain_graphql_requests_sent_to::<SlowSchema>(), @r#"
        [
          {
            "query": "query($var0: Int!) { slow: delay(ms: $var0) }",
            "operationName": null,
            "variables": {
              "var0": 100
            },
            "extensions": {}
          }
        ]
        "#);
    })
}

#[test]
fn inflight_deduplication_default() {
    runtime().block_on(async move {
        let gateway = Gateway::builder().with_subgraph(SlowSchema::default()).build().await;

        let responses = (0..10)
            .map(|_| async { gateway.post("query { slow: delay(ms: 100) }").await })
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>()
            .await;

        insta::assert_json_snapshot!(responses[0], @r#"
        {
          "data": {
            "slow": 100
          }
        }
        "#);

        insta::assert_json_snapshot!(gateway.drain_graphql_requests_sent_to::<SlowSchema>(), @r#"
        [
          {
            "query": "query($var0: Int!) { slow: delay(ms: $var0) }",
            "operationName": null,
            "variables": {
              "var0": 100
            },
            "extensions": {}
          }
        ]
        "#);
    })
}

#[test]
fn inflight_deduplication_disabled() {
    runtime().block_on(async move {
        let gateway = Gateway::builder()
            .with_subgraph(SlowSchema::default())
            .with_toml_config(
                r###"
                [traffic_shaping]
                inflight_deduplication = false
                "###,
            )
            .build()
            .await;

        let responses = (0..10)
            .map(|_| async { gateway.post("query { slow: delay(ms: 100) }").await })
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>()
            .await;

        insta::assert_json_snapshot!(responses[0], @r#"
        {
          "data": {
            "slow": 100
          }
        }
        "#);

        assert_eq!(gateway.drain_graphql_requests_sent_to::<SlowSchema>().len(), 10);
    })
}
