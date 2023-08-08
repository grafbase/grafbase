#![allow(clippy::use_self)]

use std::borrow::Cow;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP: {0}")]
    DatadogRequest(reqwest::Error),
    #[error("Datadog: [status = {0}] {1:?}")]
    DatadogPushFailed(reqwest::StatusCode, Option<String>),
}

#[derive(Clone, Copy, serde::Serialize, strum::Display, PartialEq, Eq, Ord, PartialOrd)]
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
    #[derive(Clone, Copy)]
    pub struct Config: u8 {
        const DATADOG = 0b0000_0001;
        #[cfg(feature = "with-worker")]
        const WORKER  = 0b0000_0010;
        const STDLOG  = 0b0000_0100;
    }
}

pub struct LogConfig<'a> {
    pub branch: Option<String>,
    pub datadog_api_key: Option<secrecy::SecretString>,
    pub environment: String,
    pub host_name: String,
    pub service_name: Cow<'static, str>,
    pub source_type: &'static str,
    pub trace_id: String,
    pub extra_tags: Vec<(&'static str, Cow<'a, str>)>,
    pub log_level: LogSeverity,
}
