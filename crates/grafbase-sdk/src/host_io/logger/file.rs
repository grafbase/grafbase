//! File logging functionality with support for log rotation.

use std::path::PathBuf;

use crate::{SdkError, wit};

/// A file logger that writes serialized messages to a file with optional rotation.
pub struct FileLogger(wit::FileLogger);

impl FileLogger {
    /// Creates a new file logger with the specified path and rotation strategy.
    pub fn new(path: impl Into<PathBuf>, rotation: LogRotation) -> Result<Self, SdkError> {
        let opts = wit::FileLoggerOptions {
            path: path.into().to_string_lossy().into_owned(),
            rotate: rotation.into(),
        };

        let logger = wit::FileLogger::init(&opts)?;

        Ok(Self(logger))
    }

    /// Logs a message to the file. The caller decides the encoding of the message.
    pub fn log(&self, message: &[u8]) -> Result<(), SdkError> {
        self.0.log(message)?;

        Ok(())
    }
}

/// Log rotation strategies for file logging.
pub enum LogRotation {
    /// Rotate when the log file reaches the specified size in bytes.
    Size(u64),
    /// Rotate the log file every minute.
    Minutely,
    /// Rotate the log file every hour.
    Hourly,
    /// Rotate the log file every day.
    Daily,
    /// Rotate the log file every week.
    Weekly,
    /// Rotate the log file every month.
    Monthly,
    /// Rotate the log file every year.
    Yearly,
}

impl From<LogRotation> for wit::FileLoggerRotation {
    fn from(value: LogRotation) -> Self {
        match value {
            LogRotation::Size(size) => wit::FileLoggerRotation::Size(size),
            LogRotation::Minutely => wit::FileLoggerRotation::Minutely,
            LogRotation::Hourly => wit::FileLoggerRotation::Hourly,
            LogRotation::Daily => wit::FileLoggerRotation::Daily,
            LogRotation::Weekly => wit::FileLoggerRotation::Weekly,
            LogRotation::Monthly => wit::FileLoggerRotation::Monthly,
            LogRotation::Yearly => wit::FileLoggerRotation::Yearly,
        }
    }
}
