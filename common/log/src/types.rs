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

#[derive(Clone, Copy, serde::Serialize, strum::Display, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum LogSeverity {
    Debug,
    Info,
    Error,
}

#[derive(Clone, serde::Serialize)]
pub struct LogEntry {
    pub trace_id: String,
    pub message: String,
    pub severity: LogSeverity,
    pub timestamp: wasm_timer::SystemTime,
    pub file_path: String,
    pub line_number: u32,
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

pub struct SentryConfig {
    pub api_key: String,
    pub dsn: String,
}

pub struct LogConfig {
    pub branch: Option<String>,
    pub datadog_api_key: Option<String>,
    pub environment: String,
    pub host_name: String,
    pub sentry_config: Option<SentryConfig>,
    pub service_name: &'static str,
    pub trace_id: String,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct SentryLogEntry {
    pub contents: String,
    pub request_id: String,
    pub hostname: String,
    pub environment: String,
    pub branch: Option<String>,
}
