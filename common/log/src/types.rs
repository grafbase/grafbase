#![allow(clippy::use_self)]

use std::borrow::Cow;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP: {0}")]
    DatadogRequest(reqwest::Error),
    #[error("Datadog: [status = {0}] {1:?}")]
    DatadogPushFailed(reqwest::StatusCode, Option<String>),
    #[cfg(feature = "sentry-cf-worker")]
    #[error("Sentry: {0}")]
    SentryError(sentry_cf_worker::SentryError),
}

#[derive(Clone, Copy, serde::Serialize, strum::Display, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum LogSeverity {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Clone, serde::Serialize)]
pub struct LogEntry {
    pub trace_id: String,
    pub message: String,
    pub severity: LogSeverity,
    #[serde(skip_serializing)]
    pub timestamp: wasm_timer::SystemTime,
    pub file_path: String,
    pub line_number: u32,
}

bitflags::bitflags! {
    pub struct Config: u8 {
        const DATADOG = 0b0000_0001;
        #[cfg(feature = "with-worker")]
        const WORKER  = 0b0000_0010;
        const STDLOG  = 0b0000_0100;
        #[cfg(feature = "with-sentry")]
        const SENTRY  = 0b0000_1000;
    }
}

#[cfg(feature = "with-sentry")]
#[derive(Debug, serde::Deserialize)]
pub struct SentryConfig {
    #[serde(alias = "sentry_api_key")]
    pub api_key: String,
    #[serde(alias = "sentry_dsn")]
    pub dsn: String,
}

pub struct LogConfig<'a> {
    pub branch: Option<String>,
    pub datadog_api_key: Option<String>,
    pub environment: String,
    pub host_name: String,
    #[cfg(feature = "with-sentry")]
    pub sentry_config: Option<SentryConfig>,
    pub service_name: Cow<'static, str>,
    pub source_type: &'static str,
    pub trace_id: String,
    pub extra_tags: Vec<(&'static str, Cow<'a, str>)>,
}
