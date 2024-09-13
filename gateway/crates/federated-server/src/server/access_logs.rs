use gateway_config::{AccessLogsConfig, RotateMode};
use grafbase_telemetry::otel::opentelemetry::metrics::UpDownCounter;
use rolling_logger::{RollingLogger, RotateStrategy};
use runtime_local::hooks::{AccessLogMessage, ChannelLogReceiver};
use std::io::Write;

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
