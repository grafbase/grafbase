mod hooks;
mod subgraph;

use crate::telemetry::metrics::{SumRow, METRICS_DELAY};

use super::{with_custom_gateway, with_gateway, ExponentialHistogramRow};

#[test]
fn basic() {
    with_gateway(|service_name, _, gateway, clickhouse| async move {
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
                WHERE ServiceName = ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'gql_operation_latency'
                "#,
            )
            .bind(&service_name)
            .fetch_one::<ExponentialHistogramRow>()
            .await
            .unwrap();
        insta::assert_json_snapshot!(row, @r###"
        {
          "Count": 1,
          "Attributes": {
            "gql.operation.name": "Simple",
            "gql.operation.query": "query Simple {\n  __typename\n}\n",
            "gql.operation.query_hash": "cAe1+tBRHQLrF/EO1ul4CTx+q5SB9YD+YtG3VDU6VCM=",
            "gql.operation.type": "query",
            "gql.operation.used_fields": "",
            "gql.response.status": "SUCCESS"
          }
        }
        "###);
    });
}

#[test]
fn introspection_should_not_appear_in_used_fields() {
    with_gateway(|service_name, _, gateway, clickhouse| async move {
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
                WHERE ServiceName = ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'gql_operation_latency'
                "#,
            )
            .bind(&service_name)
            .fetch_one::<ExponentialHistogramRow>()
            .await
            .unwrap();

        insta::assert_json_snapshot!(row, @r###"
        {
          "Count": 1,
          "Attributes": {
            "gql.operation.name": "__schema",
            "gql.operation.query": "query {\n  __schema {\n    description\n  }\n}\n",
            "gql.operation.query_hash": "0AmmdiLirkkd0r11qjmdCjpV7OGLe0J5c4yugMq1oeQ=",
            "gql.operation.type": "query",
            "gql.operation.used_fields": "",
            "gql.response.status": "SUCCESS"
          }
        }
        "###);
    });
}

#[test]
fn used_fields_should_be_unique() {
    with_gateway(|service_name, _, gateway, clickhouse| async move {
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
                WHERE ServiceName = ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'gql_operation_latency'
                "#,
            )
            .bind(&service_name)
            .fetch_one::<ExponentialHistogramRow>()
            .await
            .unwrap();
        insta::assert_json_snapshot!(row, @r###"
        {
          "Count": 1,
          "Attributes": {
            "gql.operation.name": "Faulty",
            "gql.operation.query": "query Faulty {\n  me {\n    id\n    reviews {\n      author {\n        id\n        username\n      }\n      body\n      body\n    }\n    username\n  }\n}\n",
            "gql.operation.query_hash": "4iL1kpGebrS0NAZQbUo76cwD4SUC5jxtUlCdc2149fg=",
            "gql.operation.type": "query",
            "gql.operation.used_fields": "User.id+username+reviews,Review.body+author,Query.me",
            "gql.response.status": "FIELD_ERROR_NULL_DATA"
          }
        }
        "###);
    });
}

#[test]
fn generate_operation_name() {
    with_gateway(|service_name, _, gateway, clickhouse| async move {
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
                WHERE ServiceName = ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'gql_operation_latency'
                "#,
            )
            .bind(&service_name)
            .fetch_one::<ExponentialHistogramRow>()
            .await
            .unwrap();

        insta::assert_json_snapshot!(row, @r###"
        {
          "Count": 1,
          "Attributes": {
            "gql.operation.name": "myFavoriteField",
            "gql.operation.query": "query {\n  ignoreMe\n  myFavoriteField\n}\n",
            "gql.operation.query_hash": "WDOyTh2uUUEIkab8iqn+MGWh5J3MntAvRkUy3yEpJS8=",
            "gql.operation.type": "query",
            "gql.operation.used_fields": "",
            "gql.response.status": "REQUEST_ERROR"
          }
        }
        "###);
    });
}

#[test]
fn request_error() {
    with_gateway(|service_name, _, gateway, clickhouse| async move {
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
                WHERE ServiceName = ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'gql_operation_latency'
                "#,
            )
            .bind(&service_name)
            .fetch_one::<ExponentialHistogramRow>()
            .await
            .unwrap();

        insta::assert_json_snapshot!(row, @r###"
        {
          "Count": 1,
          "Attributes": {
            "gql.operation.name": "Faulty",
            "gql.operation.query": "query Faulty {\n  __typ__ename\n}\n",
            "gql.operation.query_hash": "er/VMZUszb2iQhlPMx46c+flOdO8hXv8PjV1Pk/6u2A=",
            "gql.operation.type": "query",
            "gql.operation.used_fields": "",
            "gql.response.status": "REQUEST_ERROR"
          }
        }
        "###);
    });
}

#[test]
fn field_error() {
    with_gateway(|service_name, _, gateway, clickhouse| async move {
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
                WHERE ServiceName = ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'gql_operation_latency'
                "#,
            )
            .bind(&service_name)
            .fetch_one::<ExponentialHistogramRow>()
            .await
            .unwrap();

        insta::assert_json_snapshot!(row, @r###"
        {
          "Count": 1,
          "Attributes": {
            "gql.operation.name": "Faulty",
            "gql.operation.query": "query Faulty {\n  __typename\n  me {\n    id\n  }\n}\n",
            "gql.operation.query_hash": "M4bDtLPhj8uQPEFBdDWqalBphwVy7V5WPXOPHrzyikE=",
            "gql.operation.type": "query",
            "gql.operation.used_fields": "User.id,Query.me",
            "gql.response.status": "FIELD_ERROR_NULL_DATA"
          }
        }
        "###);
    });
}

#[test]
fn field_error_data_null() {
    with_gateway(|service_name, _, gateway, clickhouse| async move {
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
                WHERE ServiceName = ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'gql_operation_latency'
                "#,
            )
            .bind(&service_name)
            .fetch_one::<ExponentialHistogramRow>()
            .await
            .unwrap();

        insta::assert_json_snapshot!(row, @r###"
        {
          "Count": 1,
          "Attributes": {
            "gql.operation.name": "Faulty",
            "gql.operation.query": "query Faulty {\n  me {\n    id\n  }\n}\n",
            "gql.operation.query_hash": "Txoer8zp21WTkEG253qN503QOPQP7Pb9utIDx55IVD8=",
            "gql.operation.type": "query",
            "gql.operation.used_fields": "User.id,Query.me",
            "gql.response.status": "FIELD_ERROR_NULL_DATA"
          }
        }
        "###);
    });
}

#[test]
fn client() {
    with_gateway(|service_name, _, gateway, clickhouse| async move {
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
                WHERE ServiceName = ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'gql_operation_latency'
                "#,
            )
            .bind(&service_name)
            .fetch_optional::<ExponentialHistogramRow>()
            .await
            .unwrap();

        insta::assert_json_snapshot!(row, @r###"
        {
          "Count": 1,
          "Attributes": {
            "gql.operation.name": "SimpleQuery",
            "gql.operation.query": "query SimpleQuery {\n  __typename\n}\n",
            "gql.operation.query_hash": "qIzPxtWwHz0t+aJjvOljljbR3aGLQAA0LI5VXjW/FwQ=",
            "gql.operation.type": "query",
            "gql.operation.used_fields": "",
            "gql.response.status": "SUCCESS",
            "http.headers.x-grafbase-client-name": "test",
            "http.headers.x-grafbase-client-version": "1.0.0"
          }
        }
        "###);
    });
}

#[test]
fn cache_miss_hit() {
    with_gateway(|service_name, _, gateway, clickhouse| async move {
        let resp = gateway
            .gql::<serde_json::Value>("query SimpleQuery { __typename }")
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
                SELECT Value, Attributes
                FROM otel_metrics_sum
                WHERE ServiceName = ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'graphql.operation.cache.miss'
                "#,
            )
            .bind(&service_name)
            .fetch_optional::<SumRow>()
            .await
            .unwrap();

        insta::assert_json_snapshot!(row, @r###"
        {
          "Value": 1.0,
          "Attributes": {}
        }
        "###);

        let resp = gateway
            .gql::<serde_json::Value>("query SimpleQuery { __typename }")
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
                SELECT Value, Attributes
                FROM otel_metrics_sum
                WHERE ServiceName = ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'graphql.operation.cache.hit'
                "#,
            )
            .bind(&service_name)
            .fetch_optional::<SumRow>()
            .await
            .unwrap();

        insta::assert_json_snapshot!(row, @r###"
        {
          "Value": 1.0,
          "Attributes": {}
        }
        "###);
    });
}

#[test]
fn prepare_duration_success() {
    with_gateway(|service_name, _, gateway, clickhouse| async move {
        let resp = gateway
            .gql::<serde_json::Value>("query SimpleQuery { __typename }")
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
                WHERE ServiceName = ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'graphql.operation.prepare.duration'
                "#,
            )
            .bind(&service_name)
            .fetch_optional::<ExponentialHistogramRow>()
            .await
            .unwrap();

        insta::assert_json_snapshot!(row, @r###"
        {
          "Count": 1,
          "Attributes": {
            "graphql.document": "query SimpleQuery {\n  __typename\n}\n",
            "graphql.operation.name": "SimpleQuery",
            "graphql.operation.success": "true"
          }
        }
        "###);
    });
}

#[test]
fn prepare_duration_fail() {
    with_gateway(|service_name, _, gateway, clickhouse| async move {
        let resp = gateway
            .gql::<serde_json::Value>("query SimpleQuery { __typename")
            .send()
            .await;

        insta::assert_json_snapshot!(resp, @r###"
        {
          "errors": [
            {
              "message": " --> 1:31\n  |\n1 | query SimpleQuery { __typename\n  |                               ^---\n  |\n  = expected selection_set, selection, directive, or arguments",
              "locations": [
                {
                  "line": 1,
                  "column": 31
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

        let row = clickhouse
            .query(
                r#"
                SELECT Count, Attributes
                FROM otel_metrics_exponential_histogram
                WHERE ServiceName = ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'graphql.operation.prepare.duration'
                "#,
            )
            .bind(&service_name)
            .fetch_optional::<ExponentialHistogramRow>()
            .await
            .unwrap();

        insta::assert_json_snapshot!(row, @r###"
        {
          "Count": 1,
          "Attributes": {
            "graphql.operation.success": "false"
          }
        }
        "###);
    });
}

#[test]
fn batch() {
    with_gateway(|service_name, _, gateway, clickhouse| async move {
        let query = String::from("query SimpleQuery { __typename }");

        let resp = gateway
            .gql_batch::<serde_json::Value, _>(&[query.clone(), query])
            .send()
            .await;

        insta::assert_json_snapshot!(resp, @r###"
        [
          {
            "data": {
              "__typename": "Query"
            }
          },
          {
            "data": {
              "__typename": "Query"
            }
          }
        ]
        "###);

        tokio::time::sleep(METRICS_DELAY).await;

        let row = clickhouse
            .query(
                r#"
                SELECT Count, Attributes
                FROM otel_metrics_exponential_histogram
                WHERE ServiceName = ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'graphql.operation.batch.size'
                "#,
            )
            .bind(&service_name)
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
fn rate_limit() {
    let config = indoc::indoc! {r#"
        [gateway.rate_limit]
        storage = "redis"

        [gateway.rate_limit.global]
        limit = 1
        duration = "1s"
    "#};

    with_custom_gateway(config, |service_name, _, gateway, clickhouse| async move {
        let resp = gateway
            .gql::<serde_json::Value>("query SimpleQuery { __typename }")
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

        let query = indoc::indoc! {r#"
            SELECT Count, Attributes
            FROM otel_metrics_exponential_histogram
            WHERE ServiceName = ?
                AND ScopeName = 'grafbase'
                AND MetricName = 'grafbase.gateway.rate_limit.duration'
        "#};

        let row = clickhouse
            .query(query)
            .bind(&service_name)
            .fetch_optional::<ExponentialHistogramRow>()
            .await
            .unwrap();

        insta::assert_json_snapshot!(row, @r###"
            {
              "Count": 1,
              "Attributes": {
                "grafbase.redis.status": "SUCCESS"
              }
            }
            "###);
    });
}
