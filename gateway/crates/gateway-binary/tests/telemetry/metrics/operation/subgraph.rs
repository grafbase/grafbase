use crate::telemetry::metrics::{with_gateway, ExponentialHistogramRow, METRICS_DELAY};

#[test]
fn basic() {
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
