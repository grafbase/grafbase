use std::io::Write;

use gateway_config::{AccessLogsConfig, RotateMode};
use grafbase_telemetry::span::GRAFBASE_TARGET;
use runtime_local::hooks::{AccessLogMessage, ChannelLogReceiver};
use tracing_appender::rolling::{RollingFileAppender, Rotation};

pub(crate) fn start(config: &AccessLogsConfig, access_log_receiver: ChannelLogReceiver) -> crate::Result<()> {
    let rotation = match config.rotate {
        RotateMode::Never => Rotation::NEVER,
        RotateMode::Minutely => Rotation::MINUTELY,
        RotateMode::Hourly => Rotation::HOURLY,
        RotateMode::Daily => Rotation::DAILY,
    };

    let mut log = RollingFileAppender::builder()
        .rotation(rotation)
        .filename_prefix("access")
        .filename_suffix("log")
        .build(&config.path)
        .map_err(|e| crate::Error::InternalError(e.to_string()))?;

    tokio::task::spawn_blocking(move || {
        while let Ok(msg) = access_log_receiver.recv() {
            match msg {
                AccessLogMessage::Data(data) => {
                    if let Err(e) = log.write_all(&data) {
                        tracing::error!(target: GRAFBASE_TARGET, "error writing to access log: {e}");
                    }
                }
                AccessLogMessage::Shutdown(guard) => {
                    if let Err(e) = log.flush() {
                        tracing::error!(target: GRAFBASE_TARGET, "error flushing access log: {e}");
                    }

                    drop(guard);
                }
            }
        }
    });

    Ok(())
}
