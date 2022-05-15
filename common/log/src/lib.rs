use quick_error::quick_error;
// FIXME: To keep Clippy happy.
pub use log_;

use std::sync::atomic::{AtomicU8, Ordering};

#[cfg(feature = "with-worker")]
pub use worker;

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

#[derive(strum::Display)]
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
    }
}

pub static LOG_CONFIG: AtomicU8 = AtomicU8::new(Config::STDLOG.bits());

pub fn configure(config: Config) {
    LOG_CONFIG.store(config.bits(), Ordering::SeqCst);
}

thread_local! {
    pub static LOG_ENTRIES: std::cell::RefCell<Vec<(String, LogSeverity, String)>> =
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
    ($status:expr, $request_id:expr, $($t:tt)*) => { {
        let message = format_args!($($t)*).to_string();
        let config = $crate::Config::from_bits_truncate($crate::LOG_CONFIG.load(std::sync::atomic::Ordering::SeqCst));

        #[cfg(feature = "with-worker")] {
            if config.contains($crate::Config::WORKER) {
                $crate::print_with_worker($status, &message);
            }
        }
        if config.contains($crate::Config::STDLOG) {
            match $status {
                $crate::LogSeverity::Debug =>
                    $crate::log_::debug!("{}", message),
                $crate::LogSeverity::Info =>
                    $crate::log_::info!("{}", message),
                $crate::LogSeverity::Error =>
                    $crate::log_::error!("{}", message),
            }
        }
        if config.contains($crate::Config::DATADOG) {
            $crate::LOG_ENTRIES.with(|log_entries| log_entries
                .try_borrow_mut()
                .expect("reentrance is impossible in our single-threaded runtime")
                .push(($request_id.to_string(), $status, message)));
        }
    } }
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

#[derive(Clone, Debug, serde::Serialize)]
pub struct DatadogLogEntry {
    ddsource: String,
    ddtags: String,
    hostname: String,
    message: String,
    service: String,
    status: String,
}

pub struct LogConfig {
    pub api_key: String,
    pub service_name: &'static str,
    pub environment: String,
    pub branch: Option<String>,
}

pub fn collect_logs_to_be_pushed(
    log_config: &LogConfig,
    request_id: &str,
    request_host_name: &str,
) -> Vec<DatadogLogEntry> {
    #[rustfmt::skip]
    let mut tags = vec![
        ("request_id", request_id),
        ("environment", &log_config.environment),
    ];
    if let Some(branch) = log_config.branch.as_ref() {
        tags.push(("branch", branch.as_str()));
    }
    let tag_string = tags
        .iter()
        .map(|(lhs, rhs)| format!("{}:{}", lhs, rhs))
        .collect::<Vec<_>>()
        .join(",");

    let entries = LOG_ENTRIES.with(|log_entries| {
        log_entries
            .try_borrow_mut()
            .expect("reentrance is impossible in our single-threaded runtime")
            .iter()
            // FIXME: Replace with `Vec::drain_filter()` when it's stable.
            .filter(|(entry_request_id, _, _)| entry_request_id == request_id)
            .map(|(_, severity, message)| DatadogLogEntry {
                ddsource: "grafbase.api".to_owned(),
                ddtags: tag_string.clone(),
                hostname: request_host_name.to_owned(),
                message: message.clone(),
                service: log_config.service_name.to_owned(),
                status: severity.to_string(),
            })
            .collect::<Vec<_>>()
    });

    LOG_ENTRIES.with(|log_entries| {
        log_entries
            .try_borrow_mut()
            .expect("reentrance is impossible in our single-threaded runtime")
            .retain(|(entry_request_id, _, _)| entry_request_id != request_id)
    });

    entries
}

pub async fn push_logs_to_datadog(log_config: LogConfig, entries: &[DatadogLogEntry]) -> Result<(), Error> {
    let config = Config::from_bits_truncate(LOG_CONFIG.load(Ordering::SeqCst));
    if !config.contains(Config::DATADOG) {
        return Ok(());
    }

    const URL: &str = "https://http-intake.logs.datadoghq.com/api/v2/logs";

    let mut res = surf::post(URL)
        .header("DD-API-KEY", &log_config.api_key)
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
