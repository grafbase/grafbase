use std::{
    sync::Arc,
    time::{Duration, UNIX_EPOCH},
};

use futures_util::Future;
use indoc::formatdoc;

use crate::{clickhouse_client, runtime, Client};

const TRACE_INGESTION_DELAY: std::time::Duration = std::time::Duration::from_secs(2);

#[test]
fn no_traceparent_no_propagation() {
    with_mock_subgraph(
        "
            [telemetry.tracing.propagation]
            trace_context = false
        ",
        graphql_mocks::EchoSchema,
        |_service_name, _start, gateway, _clickhouse| async move {
            let request = r#"
                query {
                    headers {
                        name
                        value
                    }
                }
            "#;

            let response: HeadersResponse = gateway
                .gql(request)
                .header("traceparent", "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01") // should not be included
                .header("baggage", "userId=Am%C3%A9lie,serverNode=DF%2028,isProduction=false") // should not be included
                .header("x-amzn-trace-id", "Root=1-5759e988-bd862e3fe1be46a994272793;Sampled=1") // should not be included
                .send()
                .await;

            response.assert_header_names(&["accept", "content-length", "content-type"]);
        },
    );
}

#[test]
fn tracecontext_traceparent_propagation() {
    with_mock_subgraph(
        "
            [telemetry.tracing.propagation]
            trace_context = true
        ",
        graphql_mocks::EchoSchema,
        |service_name, start_time_unix, gateway, clickhouse| async move {
            let request = r#"
                query {
                    headers {
                        name
                        value
                    }
                }
            "#;

            let response: HeadersResponse = gateway
                .gql(request)
                .header("traceparent", "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01")
                .header("baggage", "userId=Am%C3%A9lie,serverNode=DF%2028,isProduction=false") // should not be included
                .header("x-amzn-trace-id", "Root=1-5759e988-bd862e3fe1be46a994272793;Sampled=1") // should not be included
                .send()
                .await;

            response.assert_header_names(&["accept", "content-length", "content-type", "traceparent", "tracestate"]);
            response.assert_header_content("tracestate", "");

            let trace_parent = response.assert_header("traceparent");

            assert_eq!(
                traceparent_deterministic_part(trace_parent),
                "00-0af7651916cd43dd8448eb211c80319c-xxxxxxxxxxxxxxxx-01"
            );

            tokio::time::sleep(TRACE_INGESTION_DELAY).await;

            let row = clickhouse
                .query(
                    r#"
                SELECT count()
                FROM otel_traces
                WHERE ServiceName = ?
                    AND Timestamp >= ?
                    AND SpanAttributes['grafbase.kind'] = 'http-request'
                    AND TraceId = '0af7651916cd43dd8448eb211c80319c'
                "#,
                )
                .bind(&service_name)
                .bind(start_time_unix)
                .fetch_one::<TracesRow>()
                .await
                .unwrap();

            insta::assert_json_snapshot!(row, @r###"
            {
              "count": 1
            }
            "###);
        },
    );
}

#[test]
fn tracecontext_and_baggage_propagation() {
    with_mock_subgraph(
        "
            [telemetry.tracing.propagation]
            trace_context = true
            baggage = true
        ",
        graphql_mocks::EchoSchema,
        |service_name, start_time_unix, gateway, clickhouse| async move {
            let request = r#"
                query {
                    headers {
                        name
                        value
                    }
                }
            "#;

            let response: HeadersResponse = gateway
                .gql(request)
                .header("traceparent", "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01")
                .header("baggage", "userId=Am%C3%A9lie,serverNode=DF%2028,isProduction=false")
                .header("x-amzn-trace-id", "Root=1-5759e988-bd862e3fe1be46a994272793;Sampled=1") // should not be included
                .send()
                .await;

            response.assert_header_names(&[
                "accept",
                "baggage",
                "content-length",
                "content-type",
                "traceparent",
                "tracestate",
            ]);
            response.assert_header_content("tracestate", "");

            let trace_parent = response.assert_header("traceparent");

            let to_deterministic_part = |traceparent: &str| {
                // https://www.w3.org/TR/trace-context/
                let mut segments = traceparent.split('-');
                let mut out = String::with_capacity(traceparent.len());

                out.push_str(segments.next().unwrap());
                out.push('-');

                out.push_str(segments.next().unwrap());
                out.push('-');

                segments.next().unwrap();
                out.push_str("xxxxxxxxxxxxxxxx");
                out.push('-');

                out.push_str(segments.next().unwrap());

                out
            };

            assert_eq!(
                to_deterministic_part(trace_parent),
                "00-0af7651916cd43dd8448eb211c80319c-xxxxxxxxxxxxxxxx-01"
            );

            let baggage = response.assert_header("baggage");
            let mut baggage_values: Vec<_> = baggage.split(',').collect();
            baggage_values.sort();
            assert_eq!(
                baggage_values,
                &["isProduction=false", "serverNode=DF%2028", "userId=Am%C3%A9lie"]
            );

            tokio::time::sleep(TRACE_INGESTION_DELAY).await;

            let row = clickhouse
                .query(
                    r#"
                SELECT count()
                FROM otel_traces
                WHERE ServiceName = ?
                    AND Timestamp >= ?
                    AND SpanAttributes['grafbase.kind'] = 'http-request'
                    AND TraceId = '0af7651916cd43dd8448eb211c80319c'
                "#,
                )
                .bind(&service_name)
                .bind(start_time_unix)
                .fetch_one::<TracesRow>()
                .await
                .unwrap();

            insta::assert_json_snapshot!(row, @r###"
            {
              "count": 1
            }
            "###);
        },
    );
}

#[test]
fn baggage_propagation() {
    with_mock_subgraph(
        "
            [telemetry.tracing.propagation]
            baggage = true
        ",
        graphql_mocks::EchoSchema,
        |_service_name, _start, gateway, _clickhouse| async move {
            let request = r#"
                query {
                    headers {
                        name
                        value
                    }
                }
            "#;

            let response: HeadersResponse = gateway
                .gql(request)
                .header("traceparent", "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01") // should not be included
                .header("baggage", "userId=Am%C3%A9lie,serverNode=DF%2028,isProduction=false")
                .header("baggage", "userName=alice") // FIXME: this should also be included (https://www.w3.org/TR/baggage/#examples-of-http-headers)
                .send()
                .await;

            response.assert_header_names(&["accept", "baggage", "content-length", "content-type"]);
            let values = response.assert_header("baggage");
            let mut values: Vec<_> = values.split(',').collect();
            values.sort();
            assert_eq!(
                values,
                &["isProduction=false", "serverNode=DF%2028", "userId=Am%C3%A9lie"]
            );
        },
    );
}

#[test]
fn aws_xray_propagation() {
    with_mock_subgraph(
        "
            [telemetry.tracing.propagation]
            trace_context = true
        ",
        graphql_mocks::EchoSchema,
        |_service_name, _start, gateway, _clickhouse| async move {
            let request = r#"
                query {
                    headers {
                        name
                        value
                    }
                }
            "#;

            let response: HeadersResponse = gateway
                .gql(request)
                .header("traceparent", "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01")
                .header("baggage", "userId=Am%C3%A9lie,serverNode=DF%2028,isProduction=false") // should not be included
                .header("x-amzn-trace-id", "Root=1-5759e988-bd862e3fe1be46a994272793;Sampled=1") // should not be included
                .send()
                .await;

            response.assert_header_names(&["accept", "content-length", "content-type", "traceparent", "tracestate"]);
            response.assert_header_content("tracestate", "");

            let trace_parent = response.assert_header("traceparent");

            assert_eq!(
                traceparent_deterministic_part(trace_parent),
                "00-0af7651916cd43dd8448eb211c80319c-xxxxxxxxxxxxxxxx-01"
            );
        },
    );
}

/// https://www.w3.org/TR/trace-context/
fn traceparent_deterministic_part(traceparent: &str) -> String {
    let mut segments = traceparent.split('-');
    let mut out = String::with_capacity(traceparent.len());

    out.push_str(segments.next().unwrap());
    out.push('-');

    out.push_str(segments.next().unwrap());
    out.push('-');

    segments.next().unwrap();
    out.push_str("xxxxxxxxxxxxxxxx");
    out.push('-');

    out.push_str(segments.next().unwrap());

    out
}

#[derive(Debug, serde::Deserialize)]
struct HeadersResponse {
    data: HeadersResponseData,
}

impl HeadersResponse {
    #[track_caller]
    fn assert_header_names(&self, expected: &[&str]) {
        let actual: Vec<_> = self.data.headers.iter().map(|h| h.name.as_str()).collect();
        assert_eq!(actual, expected);
    }

    #[track_caller]
    fn assert_header_content(&self, header_name: &str, expected_value: &str) {
        let value = self.assert_header(header_name);
        assert_eq!(value, expected_value);
    }

    #[track_caller]
    fn assert_header<'a>(&'a self, header_name: &str) -> &'a str {
        let header = self
            .data
            .headers
            .iter()
            .find(|h| h.name == header_name)
            .expect("header not found");

        &header.value
    }
}

#[derive(Debug, serde::Deserialize)]
struct HeadersResponseData {
    headers: Vec<Header>,
}

#[derive(Debug, serde::Deserialize)]
struct Header {
    name: String,
    value: String,
}

#[derive(Debug, clickhouse::Row, serde::Deserialize, serde::Serialize, PartialEq)]
struct TracesRow {
    count: u64,
}

fn with_mock_subgraph<T, F>(config: &str, subgraph_schema: impl graphql_mocks::Schema + 'static, test: T)
where
    T: FnOnce(String, u64, Arc<Client>, &'static clickhouse::Client) -> F,
    F: Future<Output = ()>,
{
    let service_name = format!("service_{}", ulid::Ulid::new());
    let config = &formatdoc! {r#"
        [graph]
        introspection = true

        [telemetry]
        service_name = "{service_name}"

        [telemetry.tracing]
        sampling = 1

        [telemetry.exporters.otlp]
        enabled = true
        endpoint = "http://localhost:4318"
        protocol = "grpc"

        [telemetry.exporters.otlp.batch_export]
        scheduled_delay = 1
        max_export_batch_size = 1

        {config}
    "#};

    let clickhouse = clickhouse_client();

    println!("service_name: {service_name}");
    println!("{config}");

    let subgraph_sdl = subgraph_schema.sdl();

    let subgraph_server = runtime().block_on(async { graphql_mocks::MockGraphQlServer::new(subgraph_schema).await });

    let federated_schema = {
        let mut subgraphs = graphql_composition::Subgraphs::default();
        subgraphs
            .ingest_str(&subgraph_sdl, "the-subgraph", subgraph_server.url().as_str())
            .unwrap();
        graphql_composition::compose(&subgraphs)
            .into_result()
            .unwrap()
            .into_federated_sdl()
    };

    crate::GatewayBuilder {
        toml_config: config.into(),
        schema: &federated_schema,
        log_level: None,
        client_url_path: None,
        client_headers: None,
    }
    .run(|client| async move {
        const WAIT_SECONDS: u64 = 2;
        let start = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + WAIT_SECONDS;

        // wait for initial polling to be pushed to OTEL tables so we can ignore it with the
        // appropriate start time filter.
        tokio::time::sleep(Duration::from_secs(WAIT_SECONDS)).await;

        test(service_name, start, client, clickhouse).await
    })
}
