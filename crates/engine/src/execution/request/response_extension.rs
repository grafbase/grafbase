use grafbase_telemetry::otel::{opentelemetry::trace::TraceContextExt, tracing_opentelemetry::OpenTelemetrySpanExt};
use schema::{AccessControl, HeaderAccessControl, Schema, SchemaConfig};

use crate::{
    prepare::PreparedOperation,
    response::{GrafbaseResponseExtension, ResponseExtensions},
};

use super::RequestContext;

pub(crate) fn should_include_grafbase_response_extension(config: &SchemaConfig, headers: &http::HeaderMap) -> bool {
    config
        .response_extension
        .access_control
        .iter()
        .any(|access_control| match access_control {
            AccessControl::Header(HeaderAccessControl {
                name,
                value: expected_value,
            }) => headers
                .get(name)
                .map(|value| {
                    if let Some(expected) = expected_value {
                        value == expected
                    } else {
                        true
                    }
                })
                .unwrap_or_default(),
            AccessControl::Deny => false,
        })
}

pub(crate) fn default_response_extensions(schema: &Schema, ctx: &RequestContext) -> ResponseExtensions {
    let mut ext = ResponseExtensions::default();
    if ctx.include_grafbase_response_extension {
        ext.grafbase = Some(if schema.config.response_extension.include_trace_id {
            let trace_id = tracing::Span::current().context().span().span_context().trace_id();
            GrafbaseResponseExtension::default().with_trace_id(trace_id)
        } else {
            GrafbaseResponseExtension::default()
        });
    }
    ext
}

pub(crate) fn response_extension_for_prepared_operation(
    schema: &Schema,
    ctx: &RequestContext,
    operation: &PreparedOperation,
) -> ResponseExtensions {
    let mut ext = default_response_extensions(schema, ctx);
    if ctx.include_grafbase_response_extension && schema.config.response_extension.include_query_plan {
        ext.grafbase = Some(ext.grafbase.unwrap_or_default().with_query_plan(schema, operation))
    };
    ext
}
