mod constants;
mod types;

use std::{
    collections::HashMap,
    sync::atomic::{AtomicU8, Ordering},
};

use futures_util as _;
pub use log_;
// Re-export.
pub use types::*;
pub use web_time;

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
pub fn print_with_worker(config: &Config, status: LogSeverity, message: &str) {
    if config.contains(Config::WORKER) {
        match status {
            LogSeverity::Trace | LogSeverity::Debug => {}
            LogSeverity::Info => worker::console_log!("{}", message),
            LogSeverity::Warn => worker::console_warn!("{}", message),
            LogSeverity::Error => worker::console_error!("{}", message),
        }
    }
}

#[cfg(not(feature = "with-worker"))]
pub fn print_with_worker(_config: &Config, _status: LogSeverity, _message: &str) {}

#[macro_export]
macro_rules! log {
    ($status:expr, $request_id:expr, $($t:tt)*) => {{
        let line_number = line!(); // must be the first line in the macro to be accurate
        let file_path = file!().to_string();

        let message = format_args!($($t)*).to_string();

        let config = $crate::Config::from_bits_truncate($crate::LOG_CONFIG.load(std::sync::atomic::Ordering::SeqCst));

        $crate::print_with_worker(&config, $status, &message);

        if config.contains($crate::Config::STDLOG) {
            match $status {
                $crate::LogSeverity::Trace => $crate::log_::trace!("{}", message),
                $crate::LogSeverity::Debug => $crate::log_::debug!("{}", message),
                $crate::LogSeverity::Info => $crate::log_::info!("{}", message),
                $crate::LogSeverity::Warn => $crate::log_::warn!("{}", message),
                $crate::LogSeverity::Error => $crate::log_::error!("{}", message),
            }
        }

        let intersection = $crate::Config::DATADOG;

        if config.intersects(intersection) {
            let should_log = $status != $crate::LogSeverity::Trace &&
                (config.contains($crate::Config::DATADOG) || $status == $crate::LogSeverity::Error);
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
                            timestamp: $crate::web_time::SystemTime::now(),
                            trace_id: $request_id.to_string(),
                        })
                });
            }
        }
    }};
}

#[macro_export]
macro_rules! trace {
    ("", $($t:tt)*) => {
        compile_error!("pass the actual trace ID here or we will leak memory.")
    };
    ($request_id:expr, $($t:tt)*) => {
        $crate::log!($crate::LogSeverity::Trace, $request_id, $($t)*)
    }
}

#[macro_export]
macro_rules! debug {
    ("", $($t:tt)*) => {
        compile_error!("pass the actual trace ID here or we will leak memory.")
    };
    ($request_id:expr, $($t:tt)*) => {
        $crate::log!($crate::LogSeverity::Debug, $request_id, $($t)*)
    }
}

#[macro_export]
macro_rules! info {
    ("", $($t:tt)*) => {
        compile_error!("pass the actual trace ID here or we will leak memory.")
    };
    ($request_id:expr, $($t:tt)*) => {
        $crate::log!($crate::LogSeverity::Info, $request_id, $($t)*)
    }
}

#[macro_export]
macro_rules! warn {
    ("", $($t:tt)*) => {
        compile_error!("pass an actual trace ID here or we will leak memory")
    };
    ($request_id:expr, $($t:tt)*) => {
        $crate::log!($crate::LogSeverity::Warn, $request_id, $($t)*)
    }
}

#[macro_export]
macro_rules! error {
    ("", $($t:tt)*) => {
        compile_error!("pass the actual trace ID here or we will leak memory.")
    };
    ($request_id:expr, $($t:tt)*) => {
        $crate::log!($crate::LogSeverity::Error, $request_id, $($t)*)
    }
}

pub fn collect_logs_to_be_pushed(log_config: &LogConfig<'_>) -> Vec<LogEntry> {
    LOG_ENTRIES.with(|log_entries| {
        let mut borrowed = log_entries
            .try_borrow_mut()
            .expect("reentrance is impossible in our single-threaded runtime");
        let entries = borrowed
            .iter()
            // FIXME: Replace with `Vec::drain_filter()` when it's stable.
            .filter(|entry| entry.trace_id == log_config.trace_id && entry.severity >= log_config.log_level)
            .cloned()
            .collect::<Vec<_>>();
        borrowed.retain(|entry| entry.trace_id != log_config.trace_id);
        entries
    })
}

pub async fn push_logs_to_datadog(log_config: &LogConfig<'_>, entries: &[LogEntry]) -> Result<(), Error> {
    use std::borrow::Cow;

    #[derive(Debug, serde::Serialize)]
    pub struct DatadogLogEntry {
        pub ddsource: String,
        pub ddtags: String,
        pub hostname: String,
        pub message: String,
        pub service: Cow<'static, str>,
        pub status: String,
    }

    if entries.is_empty() {
        return Ok(());
    }

    let Some(datadog_api_key) = log_config.datadog_api_key.as_ref() else {
        return Ok(());
    };

    // We use `Cow` to avoid needless cloning.
    let mut tags: HashMap<&'static str, Cow<'_, str>> = maplit::hashmap! {
        "request_id" => (&log_config.trace_id).into(),
        "environment" => (&log_config.environment).into(),
    };
    if let Some(branch) = log_config.branch.as_deref() {
        tags.insert("branch", Cow::Borrowed(branch));
    }
    tags.extend(log_config.extra_tags.iter().cloned());

    let entries: Vec<_> = entries
        .iter()
        .map(|entry| {
            let datadog_tag_string = {
                tags.insert("file_path", Cow::Borrowed(&entry.file_path)); // Borrowed.
                tags.insert("line_number", Cow::Owned(entry.line_number.to_string()));
                let string = tags
                    .iter()
                    .map(|(lhs, rhs)| format!("{lhs}:{rhs}"))
                    .collect::<Vec<_>>()
                    .join(",");
                string
            };

            DatadogLogEntry {
                ddsource: log_config.source_type.to_owned(),
                ddtags: datadog_tag_string,
                hostname: log_config.host_name.clone(),
                message: entry.message.clone(),
                service: log_config.service_name.clone(),
                status: entry.severity.to_string(),
            }
        })
        .collect();

    use secrecy::ExposeSecret;
    let response = reqwest::Client::new()
        .post(constants::DATADOG_INTAKE_URL)
        .header("DD-API-KEY", datadog_api_key.expose_secret())
        .json(&entries)
        .send()
        .await
        .map_err(Error::DatadogRequest)?;

    if response.status().is_success() {
        Ok(())
    } else {
        let response_status = response.status();
        let response_text = response.text().await.ok();
        Err(Error::DatadogPushFailed(response_status, response_text))
    }
}

/// [`std::dbg`] modified to use [`worker::console_debug`]
#[macro_export]
macro_rules! dbg {
    // NOTE: We cannot use `concat!` to make a static string as a format argument
    // of `eprintln!` because `file!` could contain a `{` or
    // `$val` expression could be a block (`{ .. }`), in which case the `eprintln!`
    // will be malformed.
    () => {
        #[cfg(feature = "with-worker")]
        worker::console_debug!("[{}:{}]", std::file!(), std::line!())
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                #[cfg(feature = "with-worker")]
                worker::console_debug!("[{}:{}] {} = {:#?}",
                    std::file!(), std::line!(), std::stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}
