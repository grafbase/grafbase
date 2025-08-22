use std::{collections::HashMap, time::Duration};

use indoc::{formatdoc, indoc};
use serde::Deserialize;

use crate::{load_schema, with_hybrid_server, with_static_server};

#[serde_with::serde_as]
#[derive(clickhouse::Row, Deserialize)]
struct Row {
    #[serde(rename = "ResourceAttributes")]
    #[serde_as(as = "Vec<(_, _)>")]
    resource_attributes: HashMap<String, String>,
}

#[serde_with::serde_as]
#[derive(clickhouse::Row, Deserialize)]
struct LogRowWithTraceId {
    #[serde(rename = "ResourceAttributes")]
    #[serde_as(as = "Vec<(_, _)>")]
    resource_attributes: HashMap<String, String>,
    #[serde(rename = "TraceId")]
    trace_id: String,
    #[serde(rename = "SpanId")]
    span_id: String,
    #[serde(rename = "Body")]
    body: String,
}

#[serde_with::serde_as]
#[derive(clickhouse::Row, Deserialize)]
struct LogRowWithAttributes {
    #[serde(rename = "LogAttributes")]
    #[serde_as(as = "Vec<(_, _)>")]
    log_attributes: HashMap<String, String>,
}

#[test]
fn with_otel() {
    let service_name = format!("service-{}", rand::random::<u128>());
    let config = &formatdoc! {r#"
        [telemetry]
        service_name = "{service_name}"

        [telemetry.tracing]
        sampling = 1

        [telemetry.exporters.otlp]
        enabled = true
        endpoint = "http://localhost:4327"
        protocol = "grpc"

        [telemetry.exporters.otlp.batch_export]
        scheduled_delay = "1s"
        max_export_batch_size = 1
    "#};

    let schema = load_schema("big");

    let query = indoc! {r#"
        { __typename }
    "#};

    with_static_server(config, &schema, None, None, |client| async move {
        let result: serde_json::Value = client.gql(query).send().await;
        serde_json::to_string_pretty(&result).unwrap();

        tokio::time::sleep(Duration::from_secs(3)).await;

        let client = crate::clickhouse_client();

        let Row { resource_attributes } = client
            .query("SELECT ResourceAttributes FROM otel_logs WHERE ServiceName = ?")
            .bind(&service_name)
            .fetch_one()
            .await
            .unwrap();

        let attribute = resource_attributes.get("service.name");
        assert_eq!(attribute.unwrap(), service_name.as_str());
    });
}

#[test]
fn with_otel_reload() {
    let service_name = format!("service-{}", rand::random::<u128>());
    let config = &formatdoc! {r#"
        [telemetry]
        service_name = "{service_name}"

        [telemetry.tracing]
        sampling = 1

        [telemetry.exporters.otlp]
        enabled = true
        endpoint = "http://localhost:4327"
        protocol = "grpc"

        [telemetry.exporters.otlp.batch_export]
        scheduled_delay = "1s"
        max_export_batch_size = 1
    "#};

    let schema = load_schema("big");

    let query = indoc! {r#"
        { __typename }
    "#};

    with_hybrid_server(config, "test_graph", &schema, |client, _, _| async move {
        let result: serde_json::Value = client.gql(query).send().await;
        serde_json::to_string_pretty(&result).unwrap();

        tokio::time::sleep(Duration::from_secs(3)).await;

        let client = crate::clickhouse_client();

        let Row { resource_attributes } = client
            .query("SELECT ResourceAttributes FROM otel_logs WHERE ServiceName = ?")
            .bind(&service_name)
            .fetch_one()
            .await
            .unwrap();

        let attribute = resource_attributes.get("service.name");
        assert_eq!(attribute.unwrap(), service_name.as_str());
    });
}

#[test]
fn with_otel_with_different_endpoint() {
    let service_name = format!("service-{}", rand::random::<u128>());
    let config = &formatdoc! {r#"
        [telemetry]
        service_name = "{service_name}"

        [telemetry.tracing]
        sampling = 1

        [telemetry.exporters.otlp]
        enabled = false
        endpoint = "http://localhost:6666"
        protocol = "grpc"

        [telemetry.exporters.otlp.batch_export]
        scheduled_delay = "1s"
        max_export_batch_size = 1

        [telemetry.logs.exporters.otlp]
        enabled = true
        endpoint = "http://localhost:4327"
        protocol = "grpc"

        [telemetry.logs.exporters.otlp.batch_export]
        scheduled_delay = "1s"
        max_export_batch_size = 1
    "#};

    let schema = load_schema("big");

    let query = indoc! {r#"
        { __typename }
    "#};

    with_static_server(config, &schema, None, None, |client| async move {
        let result: serde_json::Value = client.gql(query).send().await;
        serde_json::to_string_pretty(&result).unwrap();

        tokio::time::sleep(Duration::from_secs(3)).await;

        let client = crate::clickhouse_client();

        let Row { resource_attributes } = client
            .query("SELECT ResourceAttributes FROM otel_logs WHERE ServiceName = ?")
            .bind(&service_name)
            .fetch_one()
            .await
            .unwrap();

        let attribute = resource_attributes.get("service.name");
        assert_eq!(attribute.unwrap(), service_name.as_str());
    });
}

#[test]
fn logs_include_trace_id() {
    let service_name = format!("service-{}", rand::random::<u128>());
    let config = &formatdoc! {r#"
        [telemetry]
        service_name = "{service_name}"

        [telemetry.tracing]
        sampling = 1.0

        [telemetry.tracing.propagation]
        trace_context = true

        [telemetry.exporters.otlp]
        enabled = true
        endpoint = "http://localhost:4327"
        protocol = "grpc"

        [telemetry.exporters.otlp.batch_export]
        scheduled_delay = "1s"
        max_export_batch_size = 1

        [telemetry.logs.exporters.otlp]
        enabled = true
        endpoint = "http://localhost:4327" 
        protocol = "grpc"

        [telemetry.logs.exporters.otlp.batch_export]
        scheduled_delay = "1s"
        max_export_batch_size = 1
    "#};

    let schema = load_schema("big");

    let query = indoc! {r#"
        { __typename }
    "#};

    // Use GatewayBuilder to enable debug logging which will generate more log events
    use crate::GatewayBuilder;

    GatewayBuilder {
        toml_config: config.into(),
        schema: &schema,
        log_level: Some("debug".to_string()),
        client_url_path: None,
        client_headers: None,
    }
    .run(|client| async move {
        // Make a request with a specific trace parent
        let result: serde_json::Value = client
            .gql(query)
            .header("traceparent", "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01")
            .send()
            .await;

        serde_json::to_string_pretty(&result).unwrap();

        tokio::time::sleep(Duration::from_secs(3)).await;

        let clickhouse_client = crate::clickhouse_client();

        // Query ALL logs to check what's actually being generated, especially around GraphQL request time
        let log_rows: Vec<LogRowWithTraceId> = clickhouse_client
            .query("SELECT ResourceAttributes, TraceId, SpanId, Body FROM otel_logs WHERE ServiceName = ? ORDER BY Timestamp DESC LIMIT 30")
            .bind(&service_name)
            .fetch_all()
            .await
            .unwrap();

        // Debug: Print all logs for verification
        for (i, row) in log_rows.iter().enumerate() {
            println!("Log {}: TraceId='{}', Body='{}'", i + 1, row.trace_id, row.body);
        }

        // Verify we have some logs
        assert!(!log_rows.is_empty(), "Expected to find logs");

        let found_expected_trace = log_rows.iter().any(|row| {
            // Check if we found our expected trace ID
            // OpenTelemetry may format trace IDs with or without leading zeros
            row.trace_id == "af7651916cd43dd8448eb211c80319c" || 
            row.trace_id == "0af7651916cd43dd8448eb211c80319c"
        });

        if !found_expected_trace {
            println!("Found trace IDs in logs: {:?}", 
                log_rows.iter().map(|r| &r.trace_id).collect::<Vec<_>>());
        }

        assert!(found_expected_trace, "Expected to find trace ID 'af7651916cd43dd8448eb211c80319c' in logs");

        // Also verify the service name is correct
        let service_attribute = log_rows[0].resource_attributes.get("service.name");
        assert_eq!(service_attribute.unwrap(), service_name.as_str());
    });
}

#[test]
fn logs_include_span_id() {
    let service_name = format!("service-{}", rand::random::<u128>());
    let config = &formatdoc! {r#"
        [telemetry]
        service_name = "{service_name}"

        [telemetry.tracing]
        sampling = 1.0

        [telemetry.tracing.propagation]
        trace_context = true

        [telemetry.exporters.otlp]
        enabled = true
        endpoint = "http://localhost:4327"
        protocol = "grpc"

        [telemetry.exporters.otlp.batch_export]
        scheduled_delay = "1s"
        max_export_batch_size = 1

        [telemetry.logs.exporters.otlp]
        enabled = true
        endpoint = "http://localhost:4327" 
        protocol = "grpc"

        [telemetry.logs.exporters.otlp.batch_export]
        scheduled_delay = "1s"
        max_export_batch_size = 1
    "#};

    let schema = load_schema("big");

    let query = indoc! {r#"
        { __typename }
    "#};

    // Use GatewayBuilder to enable debug logging which will generate more log events
    use crate::GatewayBuilder;

    GatewayBuilder {
        toml_config: config.into(),
        schema: &schema,
        log_level: Some("debug".to_string()),
        client_url_path: None,
        client_headers: None,
    }
    .run(|client| async move {
        // Make a request with a specific trace parent
        let result: serde_json::Value = client
            .gql(query)
            .header("traceparent", "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01")
            .send()
            .await;

        serde_json::to_string_pretty(&result).unwrap();

        tokio::time::sleep(Duration::from_secs(3)).await;

        let clickhouse_client = crate::clickhouse_client();

        // Query logs with span information
        let log_rows: Vec<LogRowWithTraceId> = clickhouse_client
            .query("SELECT ResourceAttributes, TraceId, SpanId, Body FROM otel_logs WHERE ServiceName = ? ORDER BY Timestamp DESC LIMIT 30")
            .bind(&service_name)
            .fetch_all()
            .await
            .unwrap();

        // Debug: Print all logs for verification
        for (i, row) in log_rows.iter().enumerate() {
            println!("Log {}: TraceId='{}', SpanId='{}', Body='{}'", i + 1, row.trace_id, row.span_id, row.body);
        }

        // Verify we have some logs
        assert!(!log_rows.is_empty(), "Expected to find logs");

        // Check that logs with trace context also have span IDs
        for row in &log_rows {
            // If a log has a trace ID, it should also have a span ID
            if !row.trace_id.is_empty() && row.trace_id != "0" && row.trace_id != "00000000000000000000000000000000" {
                assert!(!row.span_id.is_empty(),
                    "Expected span_id to be non-empty for log with trace_id '{}', but found empty span_id. Log body: '{}'",
                    row.trace_id, row.body);

                // Span IDs should be 16 hex characters (8 bytes) or 8 hex characters (4 bytes)
                assert!(row.span_id.len() == 16 || row.span_id.len() == 8,
                    "Expected span_id to be 8 or 16 hex characters, got '{}' with length {} for log: '{}'",
                    row.span_id, row.span_id.len(), row.body);

                // Verify it's not all zeros
                assert!(row.span_id != "0000000000000000" && row.span_id != "00000000",
                    "Expected valid span_id but got all zeros for log: '{}'", row.body);
            }
        }
        // Verify we found at least some logs with both trace and span IDs
        let logs_with_context = log_rows.iter().filter(|row|
            !row.trace_id.is_empty() && !row.span_id.is_empty()
        ).count();

        assert!(logs_with_context > 0,
            "Expected to find at least some logs with both trace_id and span_id");

        // Also verify the service name is correct
        let service_attribute = log_rows[0].resource_attributes.get("service.name");
        assert_eq!(service_attribute.unwrap(), service_name.as_str());
    });
}

#[test]
fn logs_include_semantic_attributes() {
    let service_name = format!("service-{}", rand::random::<u128>());
    let config = &formatdoc! {r#"
        [telemetry]
        service_name = "{service_name}"

        [telemetry.tracing]
        sampling = 1.0

        [telemetry.tracing.propagation]
        trace_context = true

        [telemetry.exporters.otlp]
        enabled = true
        endpoint = "http://localhost:4327"
        protocol = "grpc"

        [telemetry.exporters.otlp.batch_export]
        scheduled_delay = "1s"
        max_export_batch_size = 1

        [telemetry.logs.exporters.otlp]
        enabled = true
        endpoint = "http://localhost:4327" 
        protocol = "grpc"

        [telemetry.logs.exporters.otlp.batch_export]
        scheduled_delay = "1s"
        max_export_batch_size = 1
    "#};

    let schema = load_schema("big");

    let query = indoc! {r#"
        { __typename }
    "#};

    // Use GatewayBuilder to enable debug logging which will generate more log events
    use crate::GatewayBuilder;

    GatewayBuilder {
        toml_config: config.into(),
        schema: &schema,
        log_level: Some("debug".to_string()),
        client_url_path: None,
        client_headers: None,
    }
    .run(|client| async move {
        let result: serde_json::Value = client
            .gql(query)
            .send()
            .await;

        serde_json::to_string_pretty(&result).unwrap();

        tokio::time::sleep(Duration::from_secs(3)).await;

        let clickhouse_client = crate::clickhouse_client();

        // Query logs with attributes to check semantic conventions
        let log_rows: Vec<LogRowWithAttributes> = clickhouse_client
            .query("SELECT LogAttributes FROM otel_logs WHERE ServiceName = ? AND TraceId != '' ORDER BY Timestamp DESC LIMIT 10")
            .bind(&service_name)
            .fetch_all()
            .await
            .unwrap();

        // Verify we have some logs with trace context
        assert!(!log_rows.is_empty(), "Expected to find logs with trace context");

        // Check for semantic convention attributes in at least one log
        let has_semantic_attributes = log_rows.iter().any(|row| {
            let has_code_namespace = row.log_attributes.contains_key("code.namespace");
            let has_code_filepath = row.log_attributes.contains_key("code.filepath");
            let has_code_filename = row.log_attributes.contains_key("code.filename");
            let has_code_lineno = row.log_attributes.contains_key("code.lineno");

            has_code_namespace || has_code_filepath || has_code_filename || has_code_lineno
        });

        assert!(has_semantic_attributes,
            "Expected to find logs with semantic convention attributes (code.namespace, code.filepath)");
    });
}
