use std::time::Duration;

use super::{with_gateway, ExponentialHistogramRow, SumMetricCountRow};

#[test]
fn basic() {
    with_gateway(|service_name, start_time_unix, gateway, clickhouse| async move {
        gateway.gql::<serde_json::Value>("{ __typename }").send().await;
        tokio::time::sleep(Duration::from_secs(2)).await;

        let SumMetricCountRow { value, attributes } = clickhouse
            .query(
                r#"
                SELECT Value, Attributes
                FROM otel_metrics_sum
                WHERE ServiceName = ? AND StartTimeUnix >= ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'request_count'
                    AND Attributes['gql.response.has_errors'] = ''
                "#,
            )
            .bind(&service_name)
            .bind(start_time_unix)
            .fetch_one()
            .await
            .unwrap();
        assert_eq!(value, 1.0);
        insta::assert_json_snapshot!(attributes, @r###"
        {
          "http.response.status_code": "200"
        }
        "###);
        let ExponentialHistogramRow { count, attributes } = clickhouse
            .query(
                r#"
                SELECT Count, Attributes
                FROM otel_metrics_exponential_histogram
                WHERE ServiceName = ? AND StartTimeUnix >= ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'request_latency'
                "#,
            )
            .bind(&service_name)
            .bind(start_time_unix)
            .fetch_one()
            .await
            .unwrap();
        assert_eq!(count, 1); // Initial polling also counts
        insta::assert_json_snapshot!(attributes, @"{}");

        gateway.gql::<serde_json::Value>("{ __typ__ename }").send().await;
        tokio::time::sleep(Duration::from_secs(2)).await;

        let SumMetricCountRow { value, attributes } = clickhouse
            .query(
                r#"
                SELECT Value, Attributes
                FROM otel_metrics_sum
                WHERE ServiceName = ? AND StartTimeUnix >= ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'request_count'
                    AND Attributes['gql.response.has_errors'] = 'true'
                "#,
            )
            .bind(&service_name)
            .bind(start_time_unix)
            .fetch_one()
            .await
            .unwrap();
        assert_eq!(value, 1.0);
        insta::assert_json_snapshot!(attributes, @r###"
        {
          "gql.response.has_errors": "true",
          "http.response.status_code": "200"
        }
        "###);
    });
}

#[test]
fn has_error() {
    with_gateway(|service_name, start_time_unix, gateway, clickhouse| async move {
        tokio::time::sleep(Duration::from_secs(2)).await;

        gateway.gql::<serde_json::Value>("{ __typ__ename }").send().await;
        tokio::time::sleep(Duration::from_secs(2)).await;

        let SumMetricCountRow { value, attributes } = clickhouse
            .query(
                r#"
                SELECT Value, Attributes
                FROM otel_metrics_sum
                WHERE ServiceName = ? AND StartTimeUnix >= ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'request_count'
                    AND Attributes['gql.response.has_errors'] = 'true'
                "#,
            )
            .bind(&service_name)
            .bind(start_time_unix)
            .fetch_one()
            .await
            .unwrap();
        assert!(value >= 1.0); // Initial polling also counts
        insta::assert_json_snapshot!(attributes, @r###"
        {
          "gql.response.has_errors": "true",
          "http.response.status_code": "200"
        }
        "###);
        let ExponentialHistogramRow { count, attributes } = clickhouse
            .query(
                r#"
                SELECT Count, Attributes
                FROM otel_metrics_exponential_histogram
                WHERE ServiceName = ? AND StartTimeUnix >= ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'request_latency'
                "#,
            )
            .bind(&service_name)
            .bind(start_time_unix)
            .fetch_one()
            .await
            .unwrap();
        assert!(count >= 1); // Initial polling also counts
        insta::assert_json_snapshot!(attributes, @"{}");
    });
}
