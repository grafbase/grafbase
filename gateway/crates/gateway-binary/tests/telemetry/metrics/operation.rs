use std::time::Duration;

use super::{with_gateway, ExponentialHistogramRow};

#[test]
fn basic() {
    with_gateway(|service_name, start_time_unix, gateway, clickhouse| async move {
        let response = gateway
            .gql::<serde_json::Value>("query Simple { __typename }")
            .send()
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "__typename": "Query"
          }
        }"###);
        tokio::time::sleep(Duration::from_secs(2)).await;

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
            .fetch_one::<ExponentialHistogramRow>()
            .await
            .unwrap();
        insta::assert_json_snapshot!(row, @r###"
        {
          "Count": 1,
          "Attributes": {
            "gql.operation.name": "Simple",
            "gql.operation.normalized_query": "query Simple {\n  __typename\n}\n",
            "gql.operation.normalized_query_hash": "cAe1+tBRHQLrF/EO1ul4CTx+q5SB9YD+YtG3VDU6VCM=",
            "gql.operation.type": "query",
            "gql.response.status": "SUCCESS"
          }
        }
        "###);
    });
}

#[test]
fn generate_operation_name() {
    with_gateway(|service_name, start_time_unix, gateway, clickhouse| async move {
        let response = gateway
            .gql::<serde_json::Value>("query { myFavoriteField ignoreMe }")
            .send()
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "errors": [
            {
              "message": "Query does not have a field named 'myFavoriteField'",
              "locations": [
                {
                  "line": 1,
                  "column": 9
                }
              ],
              "extensions": {
                "code": "OPERATION_VALIDATION_ERROR"
              }
            }
          ]
        }
        "###);
        tokio::time::sleep(Duration::from_secs(2)).await;

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
            .fetch_one::<ExponentialHistogramRow>()
            .await
            .unwrap();
        insta::assert_json_snapshot!(row, @r###"
        {
          "Count": 1,
          "Attributes": {
            "gql.operation.name": "myFavoriteField",
            "gql.operation.normalized_query": "query {\n  ignoreMe\n  myFavoriteField\n}\n",
            "gql.operation.normalized_query_hash": "WDOyTh2uUUEIkab8iqn+MGWh5J3MntAvRkUy3yEpJS8=",
            "gql.operation.type": "query",
            "gql.response.status": "REQUEST_ERROR"
          }
        }
        "###);
    });
}

#[test]
fn request_error() {
    with_gateway(|service_name, start_time_unix, gateway, clickhouse| async move {
        let resp = gateway
            .gql::<serde_json::Value>("query Faulty { __typ__ename }")
            .send()
            .await;
        insta::assert_json_snapshot!(resp, @r###"
        {
          "errors": [
            {
              "message": "Query does not have a field named '__typ__ename'",
              "locations": [
                {
                  "line": 1,
                  "column": 16
                }
              ],
              "extensions": {
                "code": "OPERATION_VALIDATION_ERROR"
              }
            }
          ]
        }
        "###);
        tokio::time::sleep(Duration::from_secs(2)).await;

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
            .fetch_one::<ExponentialHistogramRow>()
            .await
            .unwrap();
        insta::assert_json_snapshot!(row, @r###"
        {
          "Count": 1,
          "Attributes": {
            "gql.operation.name": "Faulty",
            "gql.operation.normalized_query": "query Faulty {\n  __typ__ename\n}\n",
            "gql.operation.normalized_query_hash": "er/VMZUszb2iQhlPMx46c+flOdO8hXv8PjV1Pk/6u2A=",
            "gql.operation.type": "query",
            "gql.response.status": "REQUEST_ERROR"
          }
        }
        "###);
    });
}

#[test]
fn field_error() {
    with_gateway(|service_name, start_time_unix, gateway, clickhouse| async move {
        let resp = gateway
            .gql::<serde_json::Value>("query Faulty { __typename me { id } }")
            .send()
            .await;
        insta::assert_json_snapshot!(resp, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "error sending request for url (http://127.0.0.1:46697/)",
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
        tokio::time::sleep(Duration::from_secs(2)).await;

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
            .fetch_one::<ExponentialHistogramRow>()
            .await
            .unwrap();
        insta::assert_json_snapshot!(row, @r###"
        {
          "Count": 1,
          "Attributes": {
            "gql.operation.name": "Faulty",
            "gql.operation.normalized_query": "query Faulty {\n  __typename\n  me {\n    id\n  }\n}\n",
            "gql.operation.normalized_query_hash": "M4bDtLPhj8uQPEFBdDWqalBphwVy7V5WPXOPHrzyikE=",
            "gql.operation.type": "query",
            "gql.response.status": "FIELD_ERROR_NULL_DATA"
          }
        }
        "###);
    });
}

#[test]
fn field_error_data_null() {
    with_gateway(|service_name, start_time_unix, gateway, clickhouse| async move {
        let resp = gateway
            .gql::<serde_json::Value>("query Faulty { me { id } }")
            .send()
            .await;
        insta::assert_json_snapshot!(resp, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "error sending request for url (http://127.0.0.1:46697/)",
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
        tokio::time::sleep(Duration::from_secs(2)).await;

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
            .fetch_one::<ExponentialHistogramRow>()
            .await
            .unwrap();
        insta::assert_json_snapshot!(row, @r###"
        {
          "Count": 1,
          "Attributes": {
            "gql.operation.name": "Faulty",
            "gql.operation.normalized_query": "query Faulty {\n  me {\n    id\n  }\n}\n",
            "gql.operation.normalized_query_hash": "Txoer8zp21WTkEG253qN503QOPQP7Pb9utIDx55IVD8=",
            "gql.operation.type": "query",
            "gql.response.status": "FIELD_ERROR_NULL_DATA"
          }
        }
        "###);
    });
}

#[test]
fn client() {
    with_gateway(|service_name, start_time_unix, gateway, clickhouse| async move {
        let resp = gateway
            .gql::<serde_json::Value>("query SimpleQuery { __typename }")
            .header("x-grafbase-client-name", "test")
            .header("x-grafbase-client-version", "1.0.0")
            .send()
            .await;
        insta::assert_json_snapshot!(resp, @r###"
        {
          "data": {
            "__typename": "Query"
          }
        }
        "###);

        tokio::time::sleep(Duration::from_secs(2)).await;

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
            "gql.response.status": "SUCCESS",
            "http.headers.x-grafbase-client-name": "test",
            "http.headers.x-grafbase-client-version": "1.0.0"
          }
        }
        "###);
    });
}
