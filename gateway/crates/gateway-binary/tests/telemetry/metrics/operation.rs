use crate::telemetry::metrics::METRICS_DELAY;

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
        tokio::time::sleep(METRICS_DELAY).await;

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
            "__grafbase.gql.operation.inferred_name": "Simple",
            "__grafbase.gql.operation.query_hash": "cAe1+tBRHQLrF/EO1ul4CTx+q5SB9YD+YtG3VDU6VCM=",
            "__grafbase.gql.operation.used_fields": "",
            "gql.operation.name": "Simple",
            "gql.operation.query": "query Simple {\n  __typename\n}\n",
            "gql.operation.type": "query",
            "gql.response.status": "SUCCESS"
          }
        }
        "###);
    });
}

#[test]
fn introspection_should_not_appear_in_used_fields() {
    with_gateway(|service_name, start_time_unix, gateway, clickhouse| async move {
        let response = gateway
            .gql::<serde_json::Value>("query { __schema { description } }")
            .send()
            .await;
        insta::assert_json_snapshot!(response, @r###"
        {
          "data": {
            "__schema": {
              "description": null
            }
          }
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
            "__grafbase.gql.operation.inferred_name": "__schema",
            "__grafbase.gql.operation.query_hash": "0AmmdiLirkkd0r11qjmdCjpV7OGLe0J5c4yugMq1oeQ=",
            "__grafbase.gql.operation.used_fields": "",
            "gql.operation.query": "query {\n  __schema {\n    description\n  }\n}\n",
            "gql.operation.type": "query",
            "gql.response.status": "SUCCESS"
          }
        }
        "###);
    });
}

#[test]
fn used_fields_should_be_unique() {
    with_gateway(|service_name, start_time_unix, gateway, clickhouse| async move {
        let resp = gateway
            .gql::<serde_json::Value>(
                r###"
                query Faulty {
                    me {
                        id
                        username
                        reviews {
                            body
                            alias: body
                            author {
                                id
                                username
                            }
                        }
                    }
                }
                "###,
            )
            .send()
            .await;
        insta::assert_json_snapshot!(resp, @r###"
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
            "__grafbase.gql.operation.inferred_name": "Faulty",
            "__grafbase.gql.operation.query_hash": "4iL1kpGebrS0NAZQbUo76cwD4SUC5jxtUlCdc2149fg=",
            "__grafbase.gql.operation.used_fields": "User.id+username+reviews,Review.body+author,Query.me",
            "gql.operation.name": "Faulty",
            "gql.operation.query": "query Faulty {\n  me {\n    id\n    reviews {\n      author {\n        id\n        username\n      }\n      body\n      body\n    }\n    username\n  }\n}\n",
            "gql.operation.type": "query",
            "gql.response.status": "FIELD_ERROR_NULL_DATA"
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
        tokio::time::sleep(METRICS_DELAY).await;

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
            "__grafbase.gql.operation.inferred_name": "myFavoriteField",
            "__grafbase.gql.operation.query_hash": "WDOyTh2uUUEIkab8iqn+MGWh5J3MntAvRkUy3yEpJS8=",
            "__grafbase.gql.operation.used_fields": "",
            "gql.operation.query": "query {\n  ignoreMe\n  myFavoriteField\n}\n",
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
        tokio::time::sleep(METRICS_DELAY).await;

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
            "__grafbase.gql.operation.inferred_name": "Faulty",
            "__grafbase.gql.operation.query_hash": "er/VMZUszb2iQhlPMx46c+flOdO8hXv8PjV1Pk/6u2A=",
            "__grafbase.gql.operation.used_fields": "",
            "gql.operation.name": "Faulty",
            "gql.operation.query": "query Faulty {\n  __typ__ename\n}\n",
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
            "__grafbase.gql.operation.inferred_name": "Faulty",
            "__grafbase.gql.operation.query_hash": "M4bDtLPhj8uQPEFBdDWqalBphwVy7V5WPXOPHrzyikE=",
            "__grafbase.gql.operation.used_fields": "User.id,Query.me",
            "gql.operation.name": "Faulty",
            "gql.operation.query": "query Faulty {\n  __typename\n  me {\n    id\n  }\n}\n",
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
            "__grafbase.gql.operation.inferred_name": "Faulty",
            "__grafbase.gql.operation.query_hash": "Txoer8zp21WTkEG253qN503QOPQP7Pb9utIDx55IVD8=",
            "__grafbase.gql.operation.used_fields": "User.id,Query.me",
            "gql.operation.name": "Faulty",
            "gql.operation.query": "query Faulty {\n  me {\n    id\n  }\n}\n",
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

        tokio::time::sleep(METRICS_DELAY).await;

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
            "__grafbase.gql.operation.inferred_name": "SimpleQuery",
            "__grafbase.gql.operation.query_hash": "qIzPxtWwHz0t+aJjvOljljbR3aGLQAA0LI5VXjW/FwQ=",
            "__grafbase.gql.operation.used_fields": "",
            "gql.operation.name": "SimpleQuery",
            "gql.operation.query": "query SimpleQuery {\n  __typename\n}\n",
            "gql.operation.type": "query",
            "gql.response.status": "SUCCESS",
            "http.headers.x-grafbase-client-name": "test",
            "http.headers.x-grafbase-client-version": "1.0.0"
          }
        }
        "###);
    });
}
