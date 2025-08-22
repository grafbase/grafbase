//! Custom OpenTelemetry logs bridge with proper trace context extraction
//!
//! This is a workaround for https://github.com/open-telemetry/opentelemetry-rust/issues/2803
//! where the standard opentelemetry-appender-tracing bridge fails to extract trace context
//! from tracing spans when used with middleware.
//!
//! ## Removal
//! This workaround can be removed when the upstream issue is resolved in
//! opentelemetry-appender-tracing > 0.30.1

use opentelemetry::{
    Key,
    logs::{AnyValue, LogRecord, Logger, LoggerProvider, Severity},
    trace::{SpanId, TraceContextExt, TraceFlags, TraceId},
};
use opentelemetry_sdk::logs::{SdkLogger, SdkLoggerProvider};
use tracing::{
    Event, Subscriber,
    field::{Field, Visit},
};
use tracing_opentelemetry::OtelData;
use tracing_subscriber::layer::Context;
use tracing_subscriber::{Layer, registry::LookupSpan};

// Production safety limits
const MAX_FIELDS_PER_EVENT: usize = 50;
const MAX_FIELD_VALUE_SIZE: usize = 1024;
const MAX_MESSAGE_SIZE: usize = 8192;

/// Extract trace context from the event span, with fallbacks
fn extract_trace_context<S>(ctx: &Context<'_, S>, event: &Event<'_>) -> Option<(TraceId, SpanId, TraceFlags)>
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    let span = ctx.event_span(event)?;
    let extensions = span.extensions();
    let otel_data = extensions.get::<OtelData>()?;
    let span_id = otel_data.builder.span_id?;

    // Try to get trace_id from parent context first, fallback to builder
    let trace_id = if otel_data.parent_cx.span().span_context().is_valid() {
        otel_data.parent_cx.span().span_context().trace_id()
    } else if let Some(trace_id) = otel_data.builder.trace_id {
        trace_id
    } else {
        // Traverse up to root span to find trace_id (like standard implementation)
        span.scope().last().and_then(|root_span| {
            root_span
                .extensions()
                .get::<OtelData>()
                .and_then(|otd| otd.builder.trace_id)
        })?
    };

    // Get trace flags from the span context or use SAMPLED as default
    let flags = if otel_data.parent_cx.span().span_context().is_valid() {
        otel_data.parent_cx.span().span_context().trace_flags()
    } else {
        TraceFlags::SAMPLED
    };

    Some((trace_id, span_id, flags))
}

const LOGGER_NAME: &str = "grafbase-gateway";

/// Custom tracing layer that properly extracts trace context and sends logs to OpenTelemetry
pub struct OtelLogsLayer {
    logger_provider: SdkLoggerProvider,
    logger: SdkLogger,
}

impl OtelLogsLayer {
    pub fn new(logger_provider: SdkLoggerProvider) -> Self {
        let logger = logger_provider.logger(LOGGER_NAME);
        Self {
            logger_provider,
            logger,
        }
    }
}

impl<S> Layer<S> for OtelLogsLayer
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        // Use the cached logger instance
        let logger = &self.logger;
        let metadata = event.metadata();

        let trace_context = extract_trace_context(&ctx, event);

        // Create OpenTelemetry log record
        let mut log_record = logger.create_log_record();

        // Note: The SDK automatically sets observed_timestamp if we don't provide one

        // Set trace context if available (this is the key fix!)
        if let Some((trace_id, span_id, flags)) = trace_context {
            log_record.set_trace_context(trace_id, span_id, Some(flags));
        }

        // Set severity level
        let severity = match *metadata.level() {
            tracing::Level::ERROR => Severity::Error,
            tracing::Level::WARN => Severity::Warn,
            tracing::Level::INFO => Severity::Info,
            tracing::Level::DEBUG => Severity::Debug,
            tracing::Level::TRACE => Severity::Trace,
        };

        log_record.set_severity_number(severity);
        log_record.set_severity_text(metadata.level().as_str());

        // Set target and event name
        log_record.set_target(metadata.target());
        log_record.set_event_name(metadata.name());

        // Extract event message and fields with safety limits
        let mut visitor = LogRecordVisitor::new();
        event.record(&mut visitor);

        // Set the log body (message) with size limit
        let message = visitor.message.unwrap_or_else(|| metadata.name().to_string());
        let truncated_message = if message.len() > MAX_MESSAGE_SIZE {
            format!("{}...[truncated]", &message[..MAX_MESSAGE_SIZE])
        } else {
            message
        };
        log_record.set_body(AnyValue::from(truncated_message));

        // Add metadata using semantic conventions
        if let Some(module_path) = metadata.module_path() {
            log_record.add_attribute(Key::new("code.namespace"), module_path);
        }

        if let Some(file) = metadata.file() {
            log_record.add_attribute(Key::new("code.filepath"), file);
            // Extract filename from path (cached in future optimization)
            let filename = file
                .rsplit_once('/')
                .map(|(_, f)| f)
                .or_else(|| file.rsplit_once('\\').map(|(_, f)| f))
                .unwrap_or(file);
            log_record.add_attribute(Key::new("code.filename"), filename);
        }

        if let Some(line) = metadata.line() {
            log_record.add_attribute(Key::new("code.lineno"), line as i64);
        }

        // Add custom fields as attributes (with limits applied)
        for (key, value) in visitor.fields.into_iter().take(MAX_FIELDS_PER_EVENT) {
            log_record.add_attribute(Key::new(key), value);
        }

        // Add field truncation warning if needed
        if visitor.fields_dropped > 0 {
            log_record.add_attribute(
                Key::new("otel.fields_dropped"),
                AnyValue::from(visitor.fields_dropped as i64),
            );
        }

        // Add any error information captured by the visitor
        if let Some(error_message) = visitor.error_message {
            log_record.add_attribute(Key::new("exception.message"), error_message);
        }

        logger.emit(log_record);
    }
}

/// Visitor to extract fields from tracing events with production safety limits
struct LogRecordVisitor {
    message: Option<String>,
    fields: Vec<(String, AnyValue)>,
    error_message: Option<AnyValue>,
    fields_dropped: usize,
}

impl LogRecordVisitor {
    fn new() -> Self {
        Self {
            message: None,
            fields: Vec::with_capacity(16), // Pre-allocate reasonable capacity
            error_message: None,
            fields_dropped: 0,
        }
    }

    fn add_field(&mut self, name: String, value: AnyValue) {
        if self.fields.len() >= MAX_FIELDS_PER_EVENT {
            self.fields_dropped += 1;
            return;
        }

        self.fields.push((name, value));
    }

    fn truncate_string(s: String) -> String {
        if s.len() > MAX_FIELD_VALUE_SIZE {
            format!("{}...[truncated]", &s[..MAX_FIELD_VALUE_SIZE])
        } else {
            s
        }
    }
}

impl Visit for LogRecordVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        let field_name = field.name();
        let formatted_value = format!("{:?}", value);

        if field_name == "message" {
            self.message = Some(Self::truncate_string(formatted_value));
        } else {
            self.add_field(
                field_name.to_string(),
                AnyValue::from(Self::truncate_string(formatted_value)),
            );
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        let field_name = field.name();

        if field_name == "message" {
            self.message = Some(Self::truncate_string(value.to_string()));
        } else {
            self.add_field(
                field_name.to_string(),
                AnyValue::from(Self::truncate_string(value.to_string())),
            );
        }
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.add_field(field.name().to_string(), AnyValue::from(value));
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.add_field(field.name().to_string(), AnyValue::from(value));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        // Safe conversion with overflow check
        let any_value = if value <= i64::MAX as u64 {
            AnyValue::from(value as i64)
        } else {
            // For values that don't fit in i64, use string representation
            AnyValue::from(value.to_string())
        };
        self.add_field(field.name().to_string(), any_value);
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.add_field(field.name().to_string(), AnyValue::from(value));
    }

    fn record_i128(&mut self, field: &Field, value: i128) {
        // Convert to string for large values that don't fit in i64
        let any_value = if let Ok(i64_value) = i64::try_from(value) {
            AnyValue::from(i64_value)
        } else {
            AnyValue::from(value.to_string())
        };
        self.add_field(field.name().to_string(), any_value);
    }

    fn record_u128(&mut self, field: &Field, value: u128) {
        // Convert to string for large values that don't fit in i64
        let any_value = if let Ok(i64_value) = i64::try_from(value) {
            AnyValue::from(i64_value)
        } else {
            AnyValue::from(value.to_string())
        };
        self.add_field(field.name().to_string(), any_value);
    }

    fn record_bytes(&mut self, field: &Field, value: &[u8]) {
        // Limit byte array size
        let truncated = if value.len() > MAX_FIELD_VALUE_SIZE {
            &value[..MAX_FIELD_VALUE_SIZE]
        } else {
            value
        };

        self.add_field(field.name().to_string(), AnyValue::from(truncated));
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        // Capture error message with size limit
        let error_str = Self::truncate_string(value.to_string());
        self.error_message = Some(AnyValue::from(error_str.clone()));
        // Also add as a field
        self.add_field(field.name().to_string(), AnyValue::from(error_str));
    }
}

impl Drop for OtelLogsLayer {
    fn drop(&mut self) {
        // Safe shutdown: use eprintln! to avoid recursion into tracing
        if let Err(err) = self.logger_provider.shutdown() {
            // Use eprintln! instead of tracing::error! to avoid infinite recursion
            eprintln!("Failed to shutdown OpenTelemetry logger provider: {}", err);
        }
    }
}
