#[macro_use]
extern crate maplit;

mod types;

// FIXME: To keep Clippy happy.
pub use log_;
use sentry_cf_worker::{send_envelope, Envelope, Event, Level, SentryError};
use std::sync::atomic::{AtomicU8, Ordering};
pub use types::*;

#[cfg(feature = "with-worker")]
pub use worker;

pub static LOG_CONFIG: AtomicU8 = AtomicU8::new(Config::STDLOG.bits());

pub static MODULE: &str = "API";

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
        if config.intersects($crate::Config::DATADOG | $crate::Config::SENTRY) {
            let should_log = config.contains($crate::Config::DATADOG) || $status == $crate::LogSeverity::Error;
            if should_log {
                $crate::LOG_ENTRIES.with(|log_entries| log_entries
                    .try_borrow_mut()
                    .expect("reentrance is impossible in our single-threaded runtime")
                    .push(($request_id.to_string(), $status, message)));
            }
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

pub fn collect_logs_to_be_pushed(
    log_config: &LogConfig,
    request_id: &str,
    request_host_name: &str,
) -> (Vec<DatadogLogEntry>, Vec<SentryLogEntry>) {
    #[rustfmt::skip]
    let mut tags = vec![
        ("request_id", request_id),
        ("environment", &log_config.environment),
    ];
    if let Some(branch) = log_config.branch.as_ref() {
        tags.push(("branch", branch.as_str()));
    }
    let datadog_tag_string = tags
        .iter()
        .map(|(lhs, rhs)| format!("{}:{}", lhs, rhs))
        .collect::<Vec<_>>()
        .join(",");

    let datadog_entries = LOG_ENTRIES.with(|log_entries| {
        log_entries
            .try_borrow_mut()
            .expect("reentrance is impossible in our single-threaded runtime")
            .iter()
            // FIXME: Replace with `Vec::drain_filter()` when it's stable.
            .filter(|(entry_request_id, _, _)| entry_request_id == request_id)
            .map(|(_, severity, message)| DatadogLogEntry {
                ddsource: "grafbase.api".to_owned(),
                ddtags: datadog_tag_string.clone(),
                hostname: request_host_name.to_owned(),
                message: message.clone(),
                service: log_config.service_name.to_owned(),
                status: severity.to_string(),
            })
            .collect::<Vec<_>>()
    });

    let sentry_entries = LOG_ENTRIES.with(|log_entries| {
        log_entries
            .try_borrow_mut()
            .expect("reentrance is impossible in our single-threaded runtime")
            .iter()
            // FIXME: Replace with `Vec::drain_filter()` when it's stable.
            .filter(|(entry_request_id, severity, _)| entry_request_id == request_id && severity == &LogSeverity::Error)
            .map(|(_, _severity, message)| SentryLogEntry {
                contents: message.clone(),
                hostname: request_host_name.to_owned(),
                request_id: request_id.to_owned(),
                environment: log_config.environment.clone(),
                branch: log_config.branch.clone(),
            })
            .collect::<Vec<_>>()
    });

    LOG_ENTRIES.with(|log_entries| {
        log_entries
            .try_borrow_mut()
            .expect("reentrance is impossible in our single-threaded runtime")
            .retain(|(entry_request_id, _, _)| entry_request_id != request_id)
    });

    (datadog_entries, sentry_entries)
}

pub async fn push_logs_to_datadog(log_config: &LogConfig, entries: &[DatadogLogEntry]) -> Result<(), Error> {
    let config = Config::from_bits_truncate(LOG_CONFIG.load(Ordering::SeqCst));
    if !config.contains(Config::DATADOG) {
        return Ok(());
    }

    if let Some(datadog_api_key) = &log_config.datadog_api_key {
        const URL: &str = "https://http-intake.logs.datadoghq.com/api/v2/logs";

        let mut res = surf::post(URL)
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
    } else {
        Ok(())
    }
}

pub fn push_logs_to_sentry(sentry_ingest_url: &str, entries: &[SentryLogEntry]) {
    let config = Config::from_bits_truncate(LOG_CONFIG.load(Ordering::SeqCst));
    if !config.contains(Config::SENTRY) || entries.is_empty() {
        return;
    }

    entries.iter().for_each(|entry| {
        let mut envelope = Envelope::new();

        let mut tags = btreemap! {
            "request_id".to_owned() => entry.request_id.clone(),
            "hostname".to_owned() => entry.hostname.clone(),
            "module".to_owned() => MODULE.to_owned(),
            "environment".to_owned() => entry.environment.clone(),
        };

        if let Some(branch) = entry.branch.as_ref() {
            tags.extend([("branch".to_owned(), branch.clone())]);
        }

        envelope.add_item(Event {
            message: Some(entry.contents.clone()),
            level: Level::Error,
            tags,
            ..Default::default()
        });

        let dsn = sentry_ingest_url.to_owned();

        worker::wasm_bindgen_futures::spawn_local(async move {
            match send_envelope(dsn, envelope).await {
                Ok(_) => {}
                Err(error) => match error {
                    SentryError::InvalidUrl => debug!("{}", "an invalid url was used for the sentry dsn"),
                    SentryError::Request(_) => debug!("{}", "a request to sentry was unsuccessful"),
                    SentryError::WriteEnvelope => debug!("{}", "could not write a sentry envelope"),
                },
            }
        });
    });
}
