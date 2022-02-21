use quick_error::quick_error;

use std::sync::atomic::AtomicBool;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        DatadogRequest(err: surf::Error) {
            display("{}", err)
        }
        DatadogPushFailed(response: String) {
            display("{}", response)
        }
    }
}

pub static ENABLE_LOGGING: AtomicBool = AtomicBool::new(false);

thread_local! {
    pub static LOG_ENTRIES: std::cell::RefCell<Vec<(String, String)>> =
        std::cell::RefCell::new(Vec::new());
}

#[macro_export]
macro_rules! debug {
    ($request_id:expr, $($t:tt)*) => {
        let message = format_args!($($t)*).to_string();
        #[cfg(feature = "worker")]
        worker::console_debug!("{}", message);
        if $crate::ENABLE_LOGGING.load(std::sync::atomic::Ordering::Relaxed) {
            $crate::LOG_ENTRIES.with(|log_entries| log_entries
                .try_borrow_mut()
                .expect("reentrance is impossible in our single-threaded runtime")
                .push(($request_id.to_string(), message)));
        }
    }
}

#[macro_export]
macro_rules! info {
    ($request_id:expr, $($t:tt)*) => {
        let message = format_args!($($t)*).to_string();
        #[cfg(feature = "worker")]
        worker::console_log!("{}", message);
        if $crate::ENABLE_LOGGING.load(std::sync::atomic::Ordering::Relaxed) {
            $crate::LOG_ENTRIES.with(|log_entries| log_entries
                .try_borrow_mut()
                .expect("reentrance is impossible in our single-threaded runtime")
                .push(($request_id.to_string(), message)));
        }
    }
}

#[macro_export]
macro_rules! error {
    ($request_id:expr, $($t:tt)*) => {
        let message = format_args!($($t)*).to_string();
        #[cfg(feature = "worker")]
        worker::console_error!("{}", message);
        if $crate::ENABLE_LOGGING.load(std::sync::atomic::Ordering::Relaxed) {
            $crate::LOG_ENTRIES.with(|log_entries| log_entries
                .try_borrow_mut()
                .expect("reentrance is impossible in our single-threaded runtime")
                .push(($request_id.to_string(), message)));
        }
    }
}

#[derive(serde::Serialize)]
pub struct DatadogLogEntry {
    ddsource: String,
    ddtags: String,
    hostname: String,
    message: String,
    service: String,
}

pub fn set_logging_enabled(enabled: bool) {
    ENABLE_LOGGING.store(enabled, std::sync::atomic::Ordering::Relaxed);
}

fn collect_logs_to_be_pushed(request_id: &str, request_host_name: &str) -> Vec<DatadogLogEntry> {
    #[rustfmt::skip]
    let tags = vec![
        ("request_id", request_id),
    ];
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
            .filter(|(entry_request_id, _)| entry_request_id == request_id)
            .map(|(_, message)| DatadogLogEntry {
                ddsource: "grafbase.api".to_owned(),
                ddtags: tag_string.clone(),
                hostname: request_host_name.to_owned(),
                message: message.clone(),
                service: "api".to_owned(),
            })
            .collect::<Vec<_>>()
    });

    LOG_ENTRIES.with(|log_entries| {
        log_entries
            .try_borrow_mut()
            .expect("reentrance is impossible in our single-threaded runtime")
            .retain(|(entry_request_id, _)| entry_request_id != request_id)
    });

    entries
}

pub async fn push_logs_to_datadog(api_key: String, request_id: String, request_host_name: String) -> Result<(), Error> {
    if !ENABLE_LOGGING.load(std::sync::atomic::Ordering::Relaxed) {
        return Ok(());
    }

    let entries = collect_logs_to_be_pushed(&request_id, &request_host_name);

    const URL: &str = "https://http-intake.logs.datadoghq.com/api/v2/logs";

    let mut res = surf::post(URL)
        .header("DD-API-KEY", api_key)
        .body_json(&entries)
        .map_err(Error::DatadogRequest)?
        .send()
        .await
        .map_err(Error::DatadogRequest)?;

    if !res.status().is_success() {
        let response = res
            .body_string()
            .await
            .expect("must be able to get the response as a string");
        return Err(Error::DatadogPushFailed(response));
    }

    Ok(())
}
