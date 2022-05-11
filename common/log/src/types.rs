use quick_error::quick_error;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        DatadogRequest(err: surf::Error) {
            display("HTTP: {err}")
        }
        DatadogPushFailed(status_code: surf::StatusCode, response: Option<String>) {
            display("Datadog: [status = {status_code}] {response:?}")
        }
    }
}

#[derive(strum::Display, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum LogSeverity {
    Debug,
    Info,
    Error,
}

bitflags::bitflags! {
    pub struct Config: u8 {
        const DATADOG = 0b00000001;
        #[cfg(feature = "with-worker")]
        const WORKER  = 0b00000010;
        const STDLOG  = 0b00000100;
        const SENTRY  = 0b00001000;
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct DatadogLogEntry {
    pub ddsource: String,
    pub ddtags: String,
    pub hostname: String,
    pub message: String,
    pub service: String,
    pub status: String,
}

pub struct LogConfig {
    pub datadog_api_key: Option<String>,
    pub service_name: &'static str,
    pub environment: String,
    pub branch: Option<String>,
    pub sentry_ingest_url: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct SentryLogEntry {
    pub contents: String,
    pub request_id: String,
    pub hostname: String,
    pub environment: String,
    pub branch: Option<String>,
}
