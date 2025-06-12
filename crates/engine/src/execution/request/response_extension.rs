use grafbase_telemetry::otel::{opentelemetry::trace::TraceContextExt, tracing_opentelemetry::OpenTelemetrySpanExt};
use schema::{AccessControl, HeaderAccessControl, Schema};

use crate::{
    engine::{Runtime, WasmExtensionContext},
    prepare::PreparedOperation,
    response::{GrafbaseResponseExtension, ResponseExtensions},
};

use super::RequestContext;

pub(crate) fn should_include_grafbase_response_extension(schema: &Schema, headers: &http::HeaderMap) -> bool {
    schema
        .settings
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

pub(crate) fn default_response_extensions<R: Runtime>(
    schema: &Schema,
    ctx: &RequestContext<WasmExtensionContext<R>>,
) -> ResponseExtensions {
    let mut ext = ResponseExtensions::default();
    if ctx.include_grafbase_response_extension {
        ext.grafbase = Some(if schema.settings.response_extension.include_trace_id {
            let trace_id = tracing::Span::current().context().span().span_context().trace_id();
            GrafbaseResponseExtension::default().with_trace_id(trace_id)
        } else {
            GrafbaseResponseExtension::default()
        });
    }
    ext
}

pub(crate) fn response_extension_for_prepared_operation<R: Runtime>(
    schema: &Schema,
    ctx: &RequestContext<WasmExtensionContext<R>>,
    operation: &PreparedOperation,
) -> ResponseExtensions {
    let mut ext = default_response_extensions::<R>(schema, ctx);
    if ctx.include_grafbase_response_extension && schema.settings.response_extension.include_query_plan {
        ext.grafbase = Some(ext.grafbase.unwrap_or_default().with_query_plan(schema, operation))
    };
    ext
}
