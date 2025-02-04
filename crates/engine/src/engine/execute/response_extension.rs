use grafbase_telemetry::otel::{opentelemetry::trace::TraceContextExt, tracing_opentelemetry::OpenTelemetrySpanExt};
use schema::{AccessControl, HeaderAccessControl};

use crate::{response::GrafbaseResponseExtension, Engine, Runtime};

use super::RequestContext;

impl<R: Runtime> Engine<R> {
    pub(crate) fn should_include_grafbase_response_extension(&self, headers: &http::HeaderMap) -> bool {
        self.schema
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

    pub(crate) fn default_grafbase_response_extension(
        &self,
        ctx: &RequestContext,
    ) -> Option<GrafbaseResponseExtension> {
        if !ctx.include_grafbase_response_extension {
            return None;
        }
        Some(if self.schema.settings.response_extension.include_trace_id {
            let trace_id = tracing::Span::current().context().span().span_context().trace_id();
            GrafbaseResponseExtension::default().with_trace_id(trace_id)
        } else {
            GrafbaseResponseExtension::default()
        })
    }
}
