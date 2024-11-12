use gateway_config::{AccessLogsConfig, RotateMode};
use grafbase_telemetry::otel::opentelemetry::metrics::UpDownCounter;
use rolling_logger::{RollingLogger, RotateStrategy};
use runtime_local::hooks::{AccessLogMessage, ChannelLogReceiver};
use std::io::Write;

/// Starts the access logging process.
///
/// This function initializes the logging mechanism based on the provided
/// configuration and begins receiving log messages from the specified
/// channel. It handles different rotation strategies for the log files
/// and ensures that logs are written correctly. The function runs in a
/// blocking task to allow asynchronous operations to continue.
///
/// # Arguments
///
/// - `config`: The configuration for the access logs, which includes
///   the path and rotation settings.
/// - `access_log_receiver`: A channel receiver to receive log messages.
/// - `pending_logs_counter`: A counter to track the number of pending
///   logs for monitoring purposes.
///
/// # Returns
///
/// This function returns a `Result` indicating success or failure. An
/// error will be returned if the logger cannot be initialized.
///
/// # Errors
///
/// This function may return an error if there are issues with the
/// logger initialization or during log writing operations.
pub(crate) fn start(
    config: &AccessLogsConfig,
    access_log_receiver: ChannelLogReceiver,
    pending_logs_counter: UpDownCounter<i64>,
) -> crate::Result<()> {
    let strategy = match config.rotate {
        RotateMode::Never => RotateStrategy::never(),
        RotateMode::Minutely => RotateStrategy::minutely(),
        RotateMode::Hourly => RotateStrategy::hourly(),
        RotateMode::Daily => RotateStrategy::daily(),
        RotateMode::Size(max_size) => RotateStrategy::size(max_size.bytes().max(0).unsigned_abs()),
    };

    let mut log = RollingLogger::new(&config.path.join("access.log"), strategy)
        .map_err(|e| crate::Error::InternalError(format!("unable to initialize access logs: {e}")))?;

    tokio::task::spawn_blocking(move || {
        while let Ok(msg) = access_log_receiver.recv() {
            pending_logs_counter.add(-1, &[]);

            match msg {
                AccessLogMessage::Data(data) => {
                    if let Err(e) = log.write_all(&data).and_then(|_| log.write(b"\n")) {
                        tracing::error!("error writing to access log: {e}");
                    }
                }
                AccessLogMessage::Shutdown(guard) => {
                    if let Err(e) = log.flush() {
                        tracing::error!("error flushing access log: {e}");
                    }

                    drop(guard);
                }
            }
        }
    });

    Ok(())
}
