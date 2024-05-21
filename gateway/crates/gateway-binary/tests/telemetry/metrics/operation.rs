use std::time::Duration;

use super::{with_gateway, ExponentialHistogramRow, SumMetricCountRow};

#[test]
fn basic() {
    with_gateway(|service_name, start_time_unix, gateway, clickhouse| async move {
        gateway
            .gql::<serde_json::Value>("query Simple { __typename }")
            .send()
            .await;
        tokio::time::sleep(Duration::from_secs(2)).await;

        let SumMetricCountRow { value, attributes } = clickhouse
            .query(
                r#"
                SELECT Value, Attributes
                FROM otel_metrics_sum
                WHERE ServiceName = ? AND StartTimeUnix >= ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'gql_operation_count'
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
          "gql.operation.name": "Simple",
          "gql.operation.normalized_query": "query Simple {\n  __typename\n}\n",
          "gql.operation.normalized_query_hash": "cAe1+tBRHQLrF/EO1ul4CTx+q5SB9YD+YtG3VDU6VCM=",
          "gql.operation.type": "query"
        }
        "###);
        let ExponentialHistogramRow { count, attributes } = clickhouse
            .query(
                r#"
                SELECT Count, Attributes
                FROM otel_metrics_exponential_histogram
                WHERE ServiceName = ? AND StartTimeUnix >= ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'gql_operation_latency'
                "#,
            )
            .bind(&service_name)
            .bind(start_time_unix)
            .fetch_one()
            .await
            .unwrap();
        assert_eq!(count, 1);
        insta::assert_json_snapshot!(attributes, @r###"
        {
          "gql.operation.name": "Simple",
          "gql.operation.normalized_query": "query Simple {\n  __typename\n}\n",
          "gql.operation.normalized_query_hash": "cAe1+tBRHQLrF/EO1ul4CTx+q5SB9YD+YtG3VDU6VCM=",
          "gql.operation.type": "query"
        }
        "###);
    });
}

#[test]
fn has_error() {
    with_gateway(|service_name, start_time_unix, gateway, clickhouse| async move {
        gateway
            .gql::<serde_json::Value>("query Simple { me { id } }")
            .send()
            .await;
        tokio::time::sleep(Duration::from_secs(2)).await;

        let SumMetricCountRow { value, attributes } = clickhouse
            .query(
                r#"
                SELECT Value, Attributes
                FROM otel_metrics_sum
                WHERE ServiceName = ? AND StartTimeUnix >= ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'gql_operation_count'
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
          "gql.operation.name": "Simple",
          "gql.operation.normalized_query": "query Simple {\n  me {\n    id\n  }\n}\n",
          "gql.operation.normalized_query_hash": "3Dn7H9sNlA2O2Wphw0R6wK0BiqcdP4oRlTiq0Ifq09M=",
          "gql.operation.type": "query",
          "gql.response.has_errors": "true"
        }
        "###);
        let ExponentialHistogramRow { count, attributes } = clickhouse
            .query(
                r#"
                SELECT Count, Attributes
                FROM otel_metrics_exponential_histogram
                WHERE ServiceName = ? AND StartTimeUnix >= ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'gql_operation_latency'
                "#,
            )
            .bind(&service_name)
            .bind(start_time_unix)
            .fetch_one()
            .await
            .unwrap();
        assert_eq!(count, 1);
        insta::assert_json_snapshot!(attributes, @r###"
        {
          "gql.operation.name": "Simple",
          "gql.operation.normalized_query": "query Simple {\n  me {\n    id\n  }\n}\n",
          "gql.operation.normalized_query_hash": "3Dn7H9sNlA2O2Wphw0R6wK0BiqcdP4oRlTiq0Ifq09M=",
          "gql.operation.type": "query",
          "gql.response.has_errors": "true"
        }
        "###);
    });
}

#[test]
fn client() {
    with_gateway(|service_name, start_time_unix, gateway, clickhouse| async move {
        gateway
            .gql::<serde_json::Value>("query SimpleQuery { __typename }")
            .header("x-grafbase-client-name", "test")
            .header("x-grafbase-client-version", "1.0.0")
            .send()
            .await;
        tokio::time::sleep(Duration::from_secs(2)).await;

        let row = clickhouse
            .query(
                r#"
                SELECT Value, Attributes
                FROM otel_metrics_sum
                WHERE ServiceName = ? AND StartTimeUnix >= ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'gql_operation_count'
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
            "gql.operation.name": "SimpleQuery",
            "gql.operation.normalized_query": "query SimpleQuery {\n  __typename\n}\n",
            "gql.operation.normalized_query_hash": "qIzPxtWwHz0t+aJjvOljljbR3aGLQAA0LI5VXjW/FwQ=",
            "gql.operation.type": "query",
            "http.headers.x-grafbase-client-name": "test",
            "http.headers.x-grafbase-client-version": "1.0.0"
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
                    AND MetricName = 'gql_operation_latency'
                "#,
            )
            .bind(&service_name)
            .bind(start_time_unix)
            .fetch_optional::<ExponentialHistogramRow>()
            .await
            .unwrap();
        insta::assert_json_snapshot!(row, @r###"
        {
          "Count": 1,
          "Attributes": {
            "gql.operation.name": "SimpleQuery",
            "gql.operation.normalized_query": "query SimpleQuery {\n  __typename\n}\n",
            "gql.operation.normalized_query_hash": "qIzPxtWwHz0t+aJjvOljljbR3aGLQAA0LI5VXjW/FwQ=",
            "gql.operation.type": "query",
            "http.headers.x-grafbase-client-name": "test",
            "http.headers.x-grafbase-client-version": "1.0.0"
          }
        }
        "###);
    });
}
