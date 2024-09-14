use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

use crate::{strategy::StrategyKind, RotateStrategy};

/// The number of seconds in a minute.
const MINUTE_SECS: u64 = 60;

/// The number of seconds in an hour.
const HOUR_SECS: u64 = MINUTE_SECS * 60;

/// The number of seconds in a day.
const DAY_SECS: u64 = HOUR_SECS * 24;

#[derive(Debug)]
/// A struct representing a log file with size, file handle, and rotation strategy.
pub(crate) struct LogFile {
    /// The current size of the log file in bytes.
    size: u64,
    /// The file handle for the log file.
    file: File,
    /// The strategy used for rotating the log file.
    rotate_strategy: RotateStrategy,
}

impl LogFile {
    /// Creates a new instance of `LogFile`.
    ///
    /// # Arguments
    ///
    /// * `path` - A reference to the `Path` where the log file will be created.
    /// * `rotate_strategy` - The strategy used for rotating the log file.
    ///
    /// # Returns
    ///
    /// Returns an `io::Result<Self>`, which is `Ok` if the log file was created successfully,
    /// or an `io::Error` if there was an issue creating the file.
    pub fn new(path: &Path, rotate_strategy: RotateStrategy) -> io::Result<Self> {
        let file = File::create(path)?;
        let size = fs::metadata(path).map_or(0, |m| m.len());

        Ok(Self {
            size,
            file,
            rotate_strategy,
        })
    }

    /// Determines if the log file needs to be rotated based on the current rotation strategy.
    ///
    /// # Returns
    ///
    /// Returns `true` if the log file meets the criteria for rotation, otherwise returns `false`.{
    pub fn needs_rotation(&self) -> bool {
        let lifetime_secs = self.rotate_strategy.lifetime().as_secs();

        match self.rotate_strategy.kind() {
            StrategyKind::Never => false,
            StrategyKind::Minutely => lifetime_secs >= MINUTE_SECS,
            StrategyKind::Hourly => lifetime_secs >= HOUR_SECS,
            StrategyKind::Daily => lifetime_secs >= DAY_SECS,
            StrategyKind::Size(max_size) => self.size >= max_size,
        }
    }

    /// Returns the timestamp of the rotation strategy in milliseconds since the epoch.
    /// The timestamp marks the point in time when the file was created.
    pub fn timestamp(&self) -> u128 {
        self.rotate_strategy.start_timestamp()
    }

    /// Returns a copy of the current `RotateStrategy` with a new rotation start time.
    pub fn copy_new_rotate(&self) -> RotateStrategy {
        self.rotate_strategy.copy_new_start()
    }

    #[cfg(test)]
    pub(crate) fn set_rotate_start(&mut self, start: std::time::SystemTime) {
        self.rotate_strategy.set_rotate_start(start);
    }
}

impl Write for LogFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.size += buf.len() as u64;
        self.file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}
