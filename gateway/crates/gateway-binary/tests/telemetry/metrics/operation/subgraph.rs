use crate::telemetry::metrics::{with_custom_gateway, with_gateway, ExponentialHistogramRow, SumRow, METRICS_DELAY};

#[test]
fn request_duration() {
    with_gateway(|service_name, start_time_unix, gateway, clickhouse| async move {
        let response = gateway
            .gql::<serde_json::Value>("query Simple { me { id } }")
            .send()
            .await;

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Request to subgraph 'accounts' failed with: error sending request for url (http://127.0.0.1:46697/)",
              "path": [
                "me"
              ],
              "extensions": {
                "code": "SUBGRAPH_REQUEST_ERROR"
              }
            }
          ]
        }
        "###);

        tokio::time::sleep(METRICS_DELAY).await;

        let row = clickhouse
            .query(
                r#"
                SELECT Count, Attributes
                FROM otel_metrics_exponential_histogram
                WHERE ServiceName = ? AND StartTimeUnix >= ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'graphql.subgraph.request.duration'
                "#,
            )
            .bind(&service_name)
            .bind(start_time_unix)
            .fetch_one::<ExponentialHistogramRow>()
            .await
            .unwrap();

        insta::assert_json_snapshot!(row, @r###"
        {
          "Count": 1,
          "Attributes": {
            "graphql.subgraph.name": "accounts",
            "graphql.subgraph.response.status": "HTTP_ERROR"
          }
        }
        "###);
    });
}

#[test]
fn request_body_size() {
    with_gateway(|service_name, start_time_unix, gateway, clickhouse| async move {
        let response = gateway
            .gql::<serde_json::Value>("query Simple { me { id } }")
            .send()
            .await;

        insta::assert_json_snapshot!(response, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Request to subgraph 'accounts' failed with: error sending request for url (http://127.0.0.1:46697/)",
              "path": [
                "me"
              ],
              "extensions": {
                "code": "SUBGRAPH_REQUEST_ERROR"
              }
            }
          ]
        }
        "###);

        tokio::time::sleep(METRICS_DELAY).await;

        let row = clickhouse
            .query(
                r#"
                SELECT Count, Attributes
                FROM otel_metrics_exponential_histogram
                WHERE ServiceName = ? AND StartTimeUnix >= ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'graphql.subgraph.request.body.size'
                "#,
            )
            .bind(&service_name)
            .bind(start_time_unix)
            .fetch_one::<ExponentialHistogramRow>()
            .await
            .unwrap();

        insta::assert_json_snapshot!(row, @r###"
        {
          "Count": 1,
          "Attributes": {
            "graphql.subgraph.name": "accounts"
          }
        }
        "###);
    });
}

#[test]
fn retries() {
    let config = indoc::indoc! {r#"
        [gateway.retry]            
        enabled = true
        min_per_second = 1
        ttl = "1s"
        retry_percent = 0.1
        retry_mutations = false
    "#};

    with_custom_gateway(
        config,
        |service_name, start_time_unix, gateway, clickhouse| async move {
            let response = gateway
                .gql::<serde_json::Value>("query Simple { me { id } }")
                .send()
                .await;

            insta::assert_json_snapshot!(response, @r###"
            {
              "data": null,
              "errors": [
                {
                  "message": "Request to subgraph 'accounts' failed with: error sending request for url (http://127.0.0.1:46697/)",
                  "path": [
                    "me"
                  ],
                  "extensions": {
                    "code": "SUBGRAPH_REQUEST_ERROR"
                  }
                }
              ]
            }
            "###);

            tokio::time::sleep(METRICS_DELAY).await;

            let rows = clickhouse
                .query(
                    r#"
                SELECT Value, Attributes
                FROM otel_metrics_sum
                WHERE ServiceName = ? AND StartTimeUnix >= ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'graphql.subgraph.request.retries'
                "#,
                )
                .bind(&service_name)
                .bind(start_time_unix)
                .fetch_all::<SumRow>()
                .await
                .unwrap();

            insta::assert_json_snapshot!(rows, @r###"
            [
              {
                "Value": 1.0,
                "Attributes": {
                  "graphql.subgraph.aborted": "false",
                  "graphql.subgraph.name": "accounts"
                }
              },
              {
                "Value": 1.0,
                "Attributes": {
                  "graphql.subgraph.aborted": "true",
                  "graphql.subgraph.name": "accounts"
                }
              }
            ]
            "###);
        },
    );
}
