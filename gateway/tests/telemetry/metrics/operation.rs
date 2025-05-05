mod hooks;
mod subgraph;

use crate::telemetry::metrics::{METRICS_DELAY, SumRow};

use super::{ExponentialHistogramRow, with_custom_gateway, with_gateway};

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
                    AND MetricName = 'graphql.operation.duration'
                "#,
            )
            .bind(&service_name)
            .fetch_one::<ExponentialHistogramRow>()
            .await
            .unwrap();
        insta::assert_json_snapshot!(row, @r#"
        {
          "Count": 1,
          "Attributes": {
            "graphql.document": "query Simple { __typename }",
            "graphql.operation.name": "Simple",
            "graphql.operation.type": "query",
            "graphql.response.status": "SUCCESS"
          }
        }
        "#);
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
                    AND MetricName = 'graphql.operation.duration'
                "#,
            )
            .bind(&service_name)
            .fetch_one::<ExponentialHistogramRow>()
            .await
            .unwrap();

        insta::assert_json_snapshot!(row, @r#"
        {
          "Count": 1,
          "Attributes": {
            "grafbase.operation.computed_name": "__schema",
            "graphql.document": "query { __schema { description } }",
            "graphql.operation.type": "query",
            "graphql.response.status": "SUCCESS"
          }
        }
        "#);
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

        insta::assert_json_snapshot!(resp, @r#"
        {
          "data": null,
          "errors": [
            {
              "message": "Request to subgraph 'accounts' failed.",
              "locations": [
                {
                  "line": 3,
                  "column": 21
                }
              ],
              "path": [
                "me"
              ],
              "extensions": {
                "code": "SUBGRAPH_REQUEST_ERROR"
              }
            }
          ]
        }
        "#);

        tokio::time::sleep(METRICS_DELAY).await;

        let row = clickhouse
            .query(
                r#"
                SELECT Count, Attributes
                FROM otel_metrics_exponential_histogram
                WHERE ServiceName = ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'graphql.operation.duration'
                "#,
            )
            .bind(&service_name)
            .fetch_one::<ExponentialHistogramRow>()
            .await
            .unwrap();
        insta::assert_json_snapshot!(row, @r#"
        {
          "Count": 1,
          "Attributes": {
            "graphql.document": "query Faulty { me { id username reviews { body alias: body author { id username } } } }",
            "graphql.operation.name": "Faulty",
            "graphql.operation.type": "query",
            "graphql.response.status": "FIELD_ERROR_NULL_DATA"
          }
        }
        "#);
    });
}

#[test]
fn generate_operation_name() {
    with_gateway(|service_name, _, gateway, clickhouse| async move {
        let response = gateway
            .gql::<serde_json::Value>("query { myFavoriteField(id: \"secret\") ignoreMe }")
            .send()
            .await;

        insta::assert_json_snapshot!(response, @r#"
        {
          "errors": [
            {
              "message": "Query does not have a field named 'myFavoriteField'.",
              "locations": [
                {
                  "line": 1,
                  "column": 9
                }
              ],
              "extensions": {
                "code": "OPERATION_VALIDATION_ERROR"
              }
            },
            {
              "message": "Query does not have a field named 'ignoreMe'.",
              "locations": [
                {
                  "line": 1,
                  "column": 39
                }
              ],
              "extensions": {
                "code": "OPERATION_VALIDATION_ERROR"
              }
            }
          ]
        }
        "#);
        tokio::time::sleep(METRICS_DELAY).await;

        let row = clickhouse
            .query(
                r#"
                SELECT Count, Attributes
                FROM otel_metrics_exponential_histogram
                WHERE ServiceName = ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'graphql.operation.duration'
                "#,
            )
            .bind(&service_name)
            .fetch_one::<ExponentialHistogramRow>()
            .await
            .unwrap();

        insta::assert_json_snapshot!(row, @r#"
        {
          "Count": 1,
          "Attributes": {
            "grafbase.operation.computed_name": "myFavoriteField",
            "graphql.document": "query { myFavoriteField(id: \"\") ignoreMe }",
            "graphql.operation.type": "query",
            "graphql.response.status": "REQUEST_ERROR"
          }
        }
        "#);
    });
}

#[test]
fn request_error() {
    with_gateway(|service_name, _, gateway, clickhouse| async move {
        let resp = gateway
            .gql::<serde_json::Value>("query Faulty { __typ__ename }")
            .send()
            .await;
        insta::assert_json_snapshot!(resp, @r#"
        {
          "errors": [
            {
              "message": "Query does not have a field named '__typ__ename'.",
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
        "#);
        tokio::time::sleep(METRICS_DELAY).await;

        let row = clickhouse
            .query(
                r#"
                SELECT Count, Attributes
                FROM otel_metrics_exponential_histogram
                WHERE ServiceName = ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'graphql.operation.duration'
                "#,
            )
            .bind(&service_name)
            .fetch_one::<ExponentialHistogramRow>()
            .await
            .unwrap();

        insta::assert_json_snapshot!(row, @r#"
        {
          "Count": 1,
          "Attributes": {
            "graphql.document": "query Faulty { __typ__ename }",
            "graphql.operation.name": "Faulty",
            "graphql.operation.type": "query",
            "graphql.response.status": "REQUEST_ERROR"
          }
        }
        "#);
    });
}

#[test]
fn field_error() {
    with_gateway(|service_name, _, gateway, clickhouse| async move {
        let resp = gateway
            .gql::<serde_json::Value>("query Faulty { __typename me { id } }")
            .send()
            .await;

        insta::assert_json_snapshot!(resp, @r#"
        {
          "data": null,
          "errors": [
            {
              "message": "Request to subgraph 'accounts' failed.",
              "locations": [
                {
                  "line": 1,
                  "column": 27
                }
              ],
              "path": [
                "me"
              ],
              "extensions": {
                "code": "SUBGRAPH_REQUEST_ERROR"
              }
            }
          ]
        }
        "#);

        tokio::time::sleep(METRICS_DELAY).await;

        let row = clickhouse
            .query(
                r#"
                SELECT Count, Attributes
                FROM otel_metrics_exponential_histogram
                WHERE ServiceName = ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'graphql.operation.duration'
                "#,
            )
            .bind(&service_name)
            .fetch_one::<ExponentialHistogramRow>()
            .await
            .unwrap();

        insta::assert_json_snapshot!(row, @r#"
        {
          "Count": 1,
          "Attributes": {
            "graphql.document": "query Faulty { __typename me { id } }",
            "graphql.operation.name": "Faulty",
            "graphql.operation.type": "query",
            "graphql.response.status": "FIELD_ERROR_NULL_DATA"
          }
        }
        "#);
    });
}

#[test]
fn field_error_data_null() {
    with_gateway(|service_name, _, gateway, clickhouse| async move {
        let resp = gateway
            .gql::<serde_json::Value>("query Faulty { me { id } }")
            .send()
            .await;

        insta::assert_json_snapshot!(resp, @r#"
        {
          "data": null,
          "errors": [
            {
              "message": "Request to subgraph 'accounts' failed.",
              "locations": [
                {
                  "line": 1,
                  "column": 16
                }
              ],
              "path": [
                "me"
              ],
              "extensions": {
                "code": "SUBGRAPH_REQUEST_ERROR"
              }
            }
          ]
        }
        "#);

        tokio::time::sleep(METRICS_DELAY).await;

        let row = clickhouse
            .query(
                r#"
                SELECT Count, Attributes
                FROM otel_metrics_exponential_histogram
                WHERE ServiceName = ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'graphql.operation.duration'
                "#,
            )
            .bind(&service_name)
            .fetch_one::<ExponentialHistogramRow>()
            .await
            .unwrap();

        insta::assert_json_snapshot!(row, @r#"
        {
          "Count": 1,
          "Attributes": {
            "graphql.document": "query Faulty { me { id } }",
            "graphql.operation.name": "Faulty",
            "graphql.operation.type": "query",
            "graphql.response.status": "FIELD_ERROR_NULL_DATA"
          }
        }
        "#);
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
                    AND MetricName = 'graphql.operation.duration'
                "#,
            )
            .bind(&service_name)
            .fetch_optional::<ExponentialHistogramRow>()
            .await
            .unwrap();

        insta::assert_json_snapshot!(row, @r#"
        {
          "Count": 1,
          "Attributes": {
            "graphql.document": "query SimpleQuery { __typename }",
            "graphql.operation.name": "SimpleQuery",
            "graphql.operation.type": "query",
            "graphql.response.status": "SUCCESS",
            "http.headers.x-grafbase-client-name": "test",
            "http.headers.x-grafbase-client-version": "1.0.0"
          }
        }
        "#);
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

        insta::assert_json_snapshot!(row, @r#"
        {
          "Count": 1,
          "Attributes": {
            "graphql.document": "query SimpleQuery { __typename }",
            "graphql.operation.name": "SimpleQuery",
            "graphql.operation.success": "true",
            "graphql.operation.type": "query"
          }
        }
        "#);
    });
}

#[test]
fn prepare_duration_fail() {
    with_gateway(|service_name, _, gateway, clickhouse| async move {
        let resp = gateway
            .gql::<serde_json::Value>("query SimpleQuery { __typename")
            .send()
            .await;

        insta::assert_json_snapshot!(resp, @r#"
        {
          "errors": [
            {
              "message": "unexpected end of file (expected one of , \":\"\"{\", \"}\", \"(\", \"@\", \"...\", RawIdent, schema, query, mutation, subscription, ty, input, true, false, null, implements, interface, \"enum\", union, scalar, extend, directive, repeatable, on, fragment)",
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
        "#);

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

#[test]
fn graphql_errors() {
    with_gateway(|service_name, _, gateway, clickhouse| async move {
        let resp = gateway
            .gql::<serde_json::Value>(
                r###"
                query Faulty {
                    me {
                        id
                    }
                }
                "###,
            )
            .send()
            .await;

        insta::assert_json_snapshot!(resp, @r#"
        {
          "data": null,
          "errors": [
            {
              "message": "Request to subgraph 'accounts' failed.",
              "locations": [
                {
                  "line": 3,
                  "column": 21
                }
              ],
              "path": [
                "me"
              ],
              "extensions": {
                "code": "SUBGRAPH_REQUEST_ERROR"
              }
            }
          ]
        }
        "#);

        tokio::time::sleep(METRICS_DELAY).await;

        let rows = clickhouse
            .query(
                r#"
                SELECT Value, Attributes
                FROM otel_metrics_sum
                WHERE ServiceName = ?
                    AND ScopeName = 'grafbase'
                    AND MetricName = 'graphql.operation.errors'
                "#,
            )
            .bind(&service_name)
            .fetch_all::<SumRow>()
            .await
            .unwrap();

        insta::assert_json_snapshot!(rows, @r###"
        [
          {
            "Value": 1.0,
            "Attributes": {
              "graphql.operation.name": "Faulty",
              "graphql.response.error.code": "SUBGRAPH_REQUEST_ERROR"
            }
          }
        ]
        "###);
    });
}
