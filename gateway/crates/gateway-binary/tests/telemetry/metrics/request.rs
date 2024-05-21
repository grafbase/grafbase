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
        insta::assert_json_snapshot!(attributes, @r###"
        {
          "http.response.status_code": "200"
        }
        "###);

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
        insta::assert_json_snapshot!(attributes, @r###"
        {
          "gql.response.has_errors": "true",
          "http.response.status_code": "200"
        }
        "###);
    });
}

#[test]
fn client() {
    with_gateway(|service_name, start_time_unix, gateway, clickhouse| async move {
        gateway
            .gql::<serde_json::Value>("{ __typename }")
            .header("x-grafbase-client-name", "test")
            .header("x-grafbase-client-version", "1.0.0")
            .send()
            .await;
        tokio::time::sleep(Duration::from_secs(2)).await;

        //
        // Unknown client
        //
        let row = clickhouse
            .query(
                r#"
                SELECT Value, Attributes
                FROM otel_metrics_sum
                WHERE ServiceName = ? AND StartTimeUnix >= ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'request_count'
                    AND Attributes['http.headers.x-grafbase-client-name'] = 'unknown'
                "#,
            )
            .bind(&service_name)
            .bind(start_time_unix)
            .fetch_optional::<SumMetricCountRow>()
            .await
            .unwrap();
        assert_eq!(row, None);
        let row = clickhouse
            .query(
                r#"
                SELECT Count, Attributes
                FROM otel_metrics_exponential_histogram
                WHERE ServiceName = ? AND StartTimeUnix >= ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'request_latency'
                    AND Attributes['http.headers.x-grafbase-client-name'] = 'unknown'
                "#,
            )
            .bind(&service_name)
            .bind(start_time_unix)
            .fetch_optional::<ExponentialHistogramRow>()
            .await
            .unwrap();
        assert_eq!(row, None);

        //
        // Unknown version
        //
        let row = clickhouse
            .query(
                r#"
                SELECT Value, Attributes
                FROM otel_metrics_sum
                WHERE ServiceName = ? AND StartTimeUnix >= ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'request_count'
                    AND Attributes['http.headers.x-grafbase-client-name'] = 'test'
                    AND Attributes['http.headers.x-grafbase-client-version'] = 'unknown'
                "#,
            )
            .bind(&service_name)
            .bind(start_time_unix)
            .fetch_optional::<SumMetricCountRow>()
            .await
            .unwrap();
        assert_eq!(row, None);
        let row = clickhouse
            .query(
                r#"
                SELECT Count, Attributes
                FROM otel_metrics_exponential_histogram
                WHERE ServiceName = ? AND StartTimeUnix >= ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'request_latency'
                    AND Attributes['http.headers.x-grafbase-client-name'] = 'test'
                    AND Attributes['http.headers.x-grafbase-client-version'] = 'unknown'
                "#,
            )
            .bind(&service_name)
            .bind(start_time_unix)
            .fetch_optional::<ExponentialHistogramRow>()
            .await
            .unwrap();
        assert_eq!(row, None);

        //
        // Known client & version
        //
        let row = clickhouse
            .query(
                r#"
                SELECT Value, Attributes
                FROM otel_metrics_sum
                WHERE ServiceName = ? AND StartTimeUnix >= ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'request_count'
                    AND Attributes['http.headers.x-grafbase-client-name'] = 'test'
                    AND Attributes['http.headers.x-grafbase-client-version'] = '1.0.0'
                "#,
            )
            .bind(&service_name)
            .bind(start_time_unix)
            .fetch_one::<SumMetricCountRow>()
            .await
            .unwrap();
        insta::assert_json_snapshot!(row, @r###"
        {
          "Value": 1.0,
          "Attributes": {
            "http.headers.x-grafbase-client-name": "test",
            "http.headers.x-grafbase-client-version": "1.0.0",
            "http.response.status_code": "200"
          }
        }
        "###);
        let row = clickhouse
            .query(
                r#"
                SELECT Count, Attributes
                FROM otel_metrics_exponential_histogram
                WHERE ServiceName = ? AND StartTimeUnix >= ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'request_latency'
                    AND Attributes['http.headers.x-grafbase-client-name'] = 'test'
                    AND Attributes['http.headers.x-grafbase-client-version'] = '1.0.0'
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
            "http.headers.x-grafbase-client-name": "test",
            "http.headers.x-grafbase-client-version": "1.0.0",
            "http.response.status_code": "200"
          }
        }
        "###);
    });
}
