use crate::telemetry::metrics::{SumRow, METRICS_DELAY};

use super::{with_gateway, ExponentialHistogramRow};

#[test]
fn basic() {
    with_gateway(|service_name, start_time_unix, gateway, clickhouse| async move {
        let resp = gateway.gql::<serde_json::Value>("{ __typename }").send().await;
        insta::assert_json_snapshot!(resp, @r###"
        {
          "data": {
            "__typename": "Query"
          }
        }
        "###);
        tokio::time::sleep(METRICS_DELAY).await;

        let mut row = clickhouse
            .query(
                r#"
                SELECT Count, Attributes
                FROM otel_metrics_exponential_histogram
                WHERE ServiceName = ? AND StartTimeUnix >= ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'http.server.request.duration'
                "#,
            )
            .bind(&service_name)
            .bind(start_time_unix)
            .fetch_one::<ExponentialHistogramRow>()
            .await
            .unwrap();

        assert!(row.attributes.contains_key("server.port"));
        row.attributes.insert("server.port".to_string(), "XXXXX".to_string());

        insta::assert_json_snapshot!(row, @r###"
        {
          "Count": 1,
          "Attributes": {
            "graphql.response.status": "SUCCESS",
            "http.request.method": "POST",
            "http.response.status.code": "200",
            "http.route": "/graphql",
            "network.protocol.version": "HTTP/1.1",
            "server.address": "127.0.0.1",
            "server.port": "XXXXX"
          }
        }
        "###);
    });
}

#[test]
fn request_error() {
    with_gateway(|service_name, start_time_unix, gateway, clickhouse| async move {
        let resp = gateway.gql::<serde_json::Value>(" __typ__ename }").send().await;
        insta::assert_json_snapshot!(resp, @r###"
        {
          "errors": [
            {
              "message": " --> 1:2\n  |\n1 |  __typ__ename }\n  |  ^---\n  |\n  = expected executable_definition",
              "locations": [
                {
                  "line": 1,
                  "column": 2
                }
              ],
              "extensions": {
                "code": "OPERATION_PARSING_ERROR"
              }
            }
          ]
        }
        "###);

        tokio::time::sleep(METRICS_DELAY).await;

        let mut row = clickhouse
            .query(
                r#"
                SELECT Count, Attributes
                FROM otel_metrics_exponential_histogram
                WHERE ServiceName = ? AND StartTimeUnix >= ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'http.server.request.duration'
                "#,
            )
            .bind(&service_name)
            .bind(start_time_unix)
            .fetch_one::<ExponentialHistogramRow>()
            .await
            .unwrap();

        assert!(row.attributes.contains_key("server.port"));
        row.attributes.insert("server.port".to_string(), "XXXXX".to_string());

        insta::assert_json_snapshot!(row, @r###"
        {
          "Count": 1,
          "Attributes": {
            "graphql.response.status": "REQUEST_ERROR",
            "http.request.method": "POST",
            "http.response.status.code": "200",
            "http.route": "/graphql",
            "network.protocol.version": "HTTP/1.1",
            "server.address": "127.0.0.1",
            "server.port": "XXXXX"
          }
        }
        "###);
    });
}

#[test]
fn field_error() {
    with_gateway(|service_name, start_time_unix, gateway, clickhouse| async move {
        let resp = gateway
            .gql::<serde_json::Value>("{ __typename me { id } }")
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

        let mut row = clickhouse
            .query(
                r#"
                SELECT Count, Attributes
                FROM otel_metrics_exponential_histogram
                WHERE ServiceName = ? AND StartTimeUnix >= ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'http.server.request.duration'
                "#,
            )
            .bind(&service_name)
            .bind(start_time_unix)
            .fetch_one::<ExponentialHistogramRow>()
            .await
            .unwrap();

        assert!(row.attributes.contains_key("server.port"));
        row.attributes.insert("server.port".to_string(), "XXXXX".to_string());

        insta::assert_json_snapshot!(row, @r###"
        {
          "Count": 1,
          "Attributes": {
            "graphql.response.status": "FIELD_ERROR_NULL_DATA",
            "http.request.method": "POST",
            "http.response.status.code": "200",
            "http.route": "/graphql",
            "network.protocol.version": "HTTP/1.1",
            "server.address": "127.0.0.1",
            "server.port": "XXXXX"
          }
        }
        "###);
    });
}

#[test]
fn field_error_data_null() {
    with_gateway(|service_name, start_time_unix, gateway, clickhouse| async move {
        let resp = gateway.gql::<serde_json::Value>("{ me { id } }").send().await;
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

        let mut row = clickhouse
            .query(
                r#"
                SELECT Count, Attributes
                FROM otel_metrics_exponential_histogram
                WHERE ServiceName = ? AND StartTimeUnix >= ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'http.server.request.duration'
                "#,
            )
            .bind(&service_name)
            .bind(start_time_unix)
            .fetch_one::<ExponentialHistogramRow>()
            .await
            .unwrap();

        assert!(row.attributes.contains_key("server.port"));
        row.attributes.insert("server.port".to_string(), "XXXXX".to_string());

        insta::assert_json_snapshot!(row, @r###"
        {
          "Count": 1,
          "Attributes": {
            "graphql.response.status": "FIELD_ERROR_NULL_DATA",
            "http.request.method": "POST",
            "http.response.status.code": "200",
            "http.route": "/graphql",
            "network.protocol.version": "HTTP/1.1",
            "server.address": "127.0.0.1",
            "server.port": "XXXXX"
          }
        }
        "###);
    });
}

#[test]
fn client() {
    with_gateway(|service_name, start_time_unix, gateway, clickhouse| async move {
        let resp = gateway
            .gql::<serde_json::Value>("{ __typename }")
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

        let mut row = clickhouse
            .query(
                r#"
                SELECT Count, Attributes
                FROM otel_metrics_exponential_histogram
                WHERE ServiceName = ? AND StartTimeUnix >= ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'http.server.request.duration'
                "#,
            )
            .bind(&service_name)
            .bind(start_time_unix)
            .fetch_optional::<ExponentialHistogramRow>()
            .await
            .unwrap()
            .unwrap();

        assert!(row.attributes.contains_key("server.port"));
        row.attributes.insert("server.port".to_string(), "XXXXX".to_string());

        insta::assert_json_snapshot!(row, @r###"
        {
          "Count": 1,
          "Attributes": {
            "graphql.response.status": "SUCCESS",
            "http.headers.x-grafbase-client-name": "test",
            "http.headers.x-grafbase-client-version": "1.0.0",
            "http.request.method": "POST",
            "http.response.status.code": "200",
            "http.route": "/graphql",
            "network.protocol.version": "HTTP/1.1",
            "server.address": "127.0.0.1",
            "server.port": "XXXXX"
          }
        }
        "###);
    });
}

#[test]
fn connected_clients() {
    with_gateway(|service_name, start_time_unix, gateway, clickhouse| async move {
        let resp = gateway.gql::<serde_json::Value>("{ __typename }").send().await;

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
                SELECT Value, Attributes
                FROM otel_metrics_sum
                WHERE ServiceName = ? AND StartTimeUnix >= ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'http.server.connected.clients'
                "#,
            )
            .bind(&service_name)
            .bind(start_time_unix)
            .fetch_optional::<SumRow>()
            .await
            .unwrap();

        insta::assert_json_snapshot!(row, @r###"
        {
          "Value": 0.0,
          "Attributes": {}
        }
        "###);
    });
}

#[test]
fn request_body_size() {
    with_gateway(|service_name, start_time_unix, gateway, clickhouse| async move {
        let resp = gateway.gql::<serde_json::Value>("{ __typename }").send().await;

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
                    AND MetricName = 'http.server.request.body.size'
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
          "Attributes": {}
        }
        "###);
    });
}

#[test]
fn response_body_size() {
    with_gateway(|service_name, start_time_unix, gateway, clickhouse| async move {
        let resp = gateway.gql::<serde_json::Value>("{ __typename }").send().await;

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
                    AND MetricName = 'http.server.response.body.size'
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
          "Attributes": {}
        }
        "###);
    });
}
