use std::io::Write;

use gateway_config::{AccessLogsConfig, RotateMode};
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
                AccessLogMessage::Data(mut data) => {
                    data.push(b'\n');

                    if let Err(e) = log.write_all(&data) {
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
