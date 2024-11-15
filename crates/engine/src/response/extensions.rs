use grafbase_telemetry::otel::opentelemetry::trace::TraceId;
use serde::Serialize;

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResponseExtensions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grafbase: Option<GrafbaseResponseExtension>,
}

impl ResponseExtensions {
    pub fn is_emtpy(&self) -> bool {
        self.grafbase.is_none()
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GrafbaseResponseExtension {
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "serialize_trace_id")]
    pub trace_id: Option<TraceId>,
}

fn serialize_trace_id<S>(trace_id: &Option<TraceId>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if let Some(trace_id) = trace_id {
        serializer.serialize_str(&format!("{trace_id:x}"))
    } else {
        serializer.serialize_none()
    }
}
