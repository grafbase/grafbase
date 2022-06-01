#[cfg(feature = "with-worker")]
#[macro_use]
extern crate maplit;

mod constants;
mod types;

// FIXME: To keep Clippy happy.
#[cfg(not(feature = "sentry-cf-worker"))]
use futures_util as _;
pub use log_;

// Re-export.
pub use types::*;
pub use wasm_timer;
#[cfg(feature = "with-worker")]
pub use worker;

use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};

pub static LOG_CONFIG: AtomicU8 = AtomicU8::new(Config::STDLOG.bits());

pub static MODULE: &str = "API";

pub fn configure(config: Config) {
    LOG_CONFIG.store(config.bits(), Ordering::SeqCst);
}

thread_local! {
    pub static LOG_ENTRIES: std::cell::RefCell<Vec<LogEntry>> =
        std::cell::RefCell::new(Vec::new());
}

#[cfg(feature = "with-worker")]
pub fn print_with_worker(status: LogSeverity, message: &str) {
    match status {
        LogSeverity::Debug => worker::console_debug!("{}", message),
        LogSeverity::Info => worker::console_log!("{}", message),
        LogSeverity::Error => worker::console_error!("{}", message),
    }
}

#[macro_export]
macro_rules! log {
    ($status:expr, $request_id:expr, $($t:tt)*) => {{
        let line_number = line!(); // must be the first line in the macro to be accurate
        let file_path = file!().to_string();

        let message = format_args!($($t)*).to_string();

        let config = $crate::Config::from_bits_truncate($crate::LOG_CONFIG.load(std::sync::atomic::Ordering::SeqCst));

        #[cfg(feature = "with-worker")]
        {
            if config.contains($crate::Config::WORKER) {
                $crate::print_with_worker($status, &message);
            }
        }

        if config.contains($crate::Config::STDLOG) {
            match $status {
                $crate::LogSeverity::Debug => $crate::log_::debug!("{}", message),
                $crate::LogSeverity::Info => $crate::log_::info!("{}", message),
                $crate::LogSeverity::Error => $crate::log_::error!("{}", message),
            }
        }
        if config.intersects($crate::Config::DATADOG | $crate::Config::SENTRY) {
            let should_log = config.contains($crate::Config::DATADOG) || $status == $crate::LogSeverity::Error;
            if should_log {
                $crate::LOG_ENTRIES.with(|log_entries| {
                    log_entries
                        .try_borrow_mut()
                        .expect("reentrance is impossible in our single-threaded runtime")
                        .push($crate::LogEntry {
                            file_path,
                            line_number,
                            message,
                            severity: $status,
                            timestamp: $crate::wasm_timer::SystemTime::now(),
                            trace_id: $request_id.to_string(),
                        })
                });
            }
        }
    }};
}

#[macro_export]
macro_rules! debug {
    ($request_id:expr, $($t:tt)*) => {
        $crate::log!($crate::LogSeverity::Debug, $request_id, $($t)*)
    }
}

#[macro_export]
macro_rules! info {
    ($request_id:expr, $($t:tt)*) => {
        $crate::log!($crate::LogSeverity::Info, $request_id, $($t)*)
    }
}

#[macro_export]
macro_rules! error {
    ($request_id:expr, $($t:tt)*) => {
        $crate::log!($crate::LogSeverity::Error, $request_id, $($t)*)
    }
}

pub fn collect_logs_to_be_pushed(log_config: &LogConfig) -> Vec<LogEntry> {
    LOG_ENTRIES.with(|log_entries| {
        let mut borrowed = log_entries
            .try_borrow_mut()
            .expect("reentrance is impossible in our single-threaded runtime");
        let entries = borrowed
            .iter()
            // FIXME: Replace with `Vec::drain_filter()` when it's stable.
            .filter(|entry| entry.trace_id == log_config.trace_id)
            .cloned()
            .collect::<Vec<_>>();
        borrowed.retain(|entry| entry.trace_id != log_config.trace_id);
        entries
    })
}

pub async fn push_logs_to_datadog(log_config: &LogConfig, entries: &[LogEntry]) -> Result<(), Error> {
    use std::borrow::Cow;

    #[derive(Debug, serde::Serialize)]
    pub struct DatadogLogEntry {
        pub ddsource: String,
        pub ddtags: String,
        pub hostname: String,
        pub message: String,
        pub service: String,
        pub status: String,
    }

    if entries.is_empty() {
        return Ok(());
    }

    let datadog_api_key = match log_config.datadog_api_key.as_deref() {
        Some(api_key) => api_key,
        None => return Ok(()),
    };

    // We use `Cow` to avoid needless cloning.
    let mut tags: HashMap<&'static str, Cow<'_, str>> = maplit::hashmap! {
        "request_id" => (&log_config.trace_id).into(),
        "environment" => (&log_config.environment).into(),
    };
    if let Some(branch) = log_config.branch.as_deref() {
        tags.insert("branch", Cow::Borrowed(branch));
    }

    let entries: Vec<_> = entries
        .iter()
        .map(|entry| {
            let datadog_tag_string = {
                tags.insert("file_path", Cow::Borrowed(&entry.file_path)); // Borrowed.
                tags.insert("line_number", Cow::Owned(entry.line_number.to_string()));
                let string = tags
                    .iter()
                    .map(|(lhs, rhs)| format!("{}:{}", lhs, rhs))
                    .collect::<Vec<_>>()
                    .join(",");
                string
            };

            DatadogLogEntry {
                ddsource: "grafbase.api".to_owned(),
                ddtags: datadog_tag_string,
                hostname: log_config.host_name.to_owned(),
                message: entry.message.clone(),
                service: log_config.service_name.to_owned(),
                status: entry.severity.to_string(),
            }
        })
        .collect();

    let mut res = surf::post(constants::DATADOG_INTAKE_URL)
        .header("DD-API-KEY", datadog_api_key)
        .body_json(&entries)
        .map_err(Error::DatadogRequest)?
        .send()
        .await
        .map_err(Error::DatadogRequest)?;

    if res.status().is_success() {
        Ok(())
    } else {
        let response = res.body_string().await.ok();
        Err(Error::DatadogPushFailed(res.status(), response))
    }
}

#[cfg(feature = "sentry-cf-worker")]
pub async fn push_logs_to_sentry(log_config: &LogConfig, entries: &[LogEntry]) -> Result<(), Error> {
    use sentry_cf_worker::{send_envelope, Envelope, Event, Level};

    let sentry_config = match log_config.sentry_config.as_ref() {
        Some(sentry_config) => sentry_config,
        None => return Ok(()),
    };

    let sentry_ingest_url = format!("https://{}@{}", sentry_config.api_key, sentry_config.dsn);

    let futures = entries
        .iter()
        .filter(|entry| entry.severity == LogSeverity::Error)
        .map(|entry| {
            let mut envelope = Envelope::new();
            let mut tags = btreemap! {
                "environment".to_owned() => log_config.environment.clone(),
                "file_path".to_owned() => entry.file_path.clone(),
                "hostname".to_owned() => log_config.host_name.clone(),
                "line_number".to_owned() => entry.line_number.to_string(),
                "module".to_owned() => MODULE.to_owned(),
                "request_id".to_owned() => entry.trace_id.clone(),
            };
            if let Some(branch) = log_config.branch.as_ref() {
                tags.extend([("branch".to_owned(), branch.clone())]);
            }
            envelope.add_item(Event {
                message: Some(entry.message.clone()),
                level: Level::Error,
                timestamp: entry.timestamp,
                tags,
                ..Default::default()
            });
            envelope
        })
        .map(|envelope| async {
            let dsn = sentry_ingest_url.clone();
            send_envelope(dsn, envelope).await
        });

    futures_util::future::try_join_all(futures)
        .await
        .map(|_| ())
        .map_err(Error::SentryError)
}
