use std::{io::Write, path::Path, sync::Arc};

use crossbeam::sync::WaitGroup;
use rolling_logger::RotateStrategy;

pub struct Inner {
    sender: crossbeam::channel::Sender<LogMessage>,
    _logger_task: tokio::task::JoinHandle<()>,
}

/// A thread-safe file logger that supports log rotation.
///
/// This logger runs in a separate blocking task and communicates via channels
/// to avoid blocking async operations. It supports graceful shutdown and
/// reference counting to determine when it's safe to drop.
#[derive(Clone)]
pub struct FileLogger {
    inner: Arc<Inner>,
}

impl FileLogger {
    /// Creates a new file logger with the specified path and rotation strategy.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path where logs will be written
    /// * `rotate` - The rotation strategy to use for log files
    ///
    /// # Returns
    ///
    /// Returns `Ok(FileLogger)` on success, or `Err(String)` if the logger
    /// could not be initialized.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rolling_logger::RotateStrategy;
    ///
    /// let logger = FileLogger::new("app.log", RotateStrategy::Size(1024 * 1024))?;
    /// # Ok::<(), String>(())
    /// ```
    pub fn new(path: impl AsRef<Path>, rotate: RotateStrategy) -> Result<Self, String> {
        let (sender, receiver) = crossbeam::channel::unbounded();

        let mut logger = rolling_logger::RollingLogger::new(path, rotate).map_err(|e| e.to_string())?;

        let logger_task = tokio::task::spawn_blocking(move || {
            while let Ok(message) = receiver.recv() {
                match message {
                    LogMessage::Data(data) => {
                        if let Err(err) = logger.write_all(&data).and_then(|_| logger.write(b"\n")) {
                            tracing::error!("Error writing log data: {err}");
                        }
                    }
                    LogMessage::Shutdown(guard) => {
                        if let Err(e) = logger.flush() {
                            tracing::error!("Error flushing log data: {e}");
                        }

                        drop(guard);
                    }
                }
            }
        });

        Ok(Self {
            inner: Arc::new(Inner {
                sender,
                _logger_task: logger_task,
            }),
        })
    }

    /// Sends data to be logged.
    ///
    /// The data will be written to the log file asynchronously in a separate task.
    /// A newline character is automatically appended after the data.
    ///
    /// # Arguments
    ///
    /// * `data` - The bytes to write to the log file
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the data was successfully queued for writing,
    /// or `Err(String)` if the logger task has been shut down.
    pub fn send(&self, data: Vec<u8>) -> Result<(), String> {
        self.inner
            .sender
            .send(LogMessage::Data(data))
            .map_err(|e| e.to_string())
    }

    /// Performs a graceful shutdown of the logger.
    ///
    /// This method sends a shutdown signal to the logger task and waits
    /// for all pending log data to be flushed to disk before returning.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), String> {
    /// let logger = FileLogger::new("app.log", rolling_logger::RotateStrategy::Never)?;
    /// logger.send(b"final log message".to_vec())?;
    /// logger.graceful_shutdown().await;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn graceful_shutdown(&self) {
        if Arc::strong_count(&self.inner) > 1 {
            tracing::debug!("logger has multiple references, cannot gracefully shutdown");
            return;
        }

        let wg = WaitGroup::new();

        if self.inner.sender.send(LogMessage::Shutdown(wg.clone())).is_err() {
            tracing::debug!("access log receiver is already dead, cannot empty log channel");
        }

        tokio::task::spawn_blocking(|| wg.wait()).await.unwrap();
    }
}

/// Messages sent to the logger task.
///
/// This enum represents the different types of messages that can be sent
/// to the background logging task via the channel.
pub enum LogMessage {
    /// A data message containing bytes to be written to the log file.
    Data(Vec<u8>),
    /// A shutdown message that signals the logger to flush and terminate.
    ///
    /// The `WaitGroup` is used to synchronize the shutdown process.
    Shutdown(WaitGroup),
}
