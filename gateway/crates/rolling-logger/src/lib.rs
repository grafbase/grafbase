//! This module provides a rolling logger that can rotate log files based on
//! different strategies such as file size or time duration. It includes a
//! `RollingLogger` struct that implements the `Write` trait, allowing it to
//! be used as a standard writer.
//!
//! The `RollingLogger` struct contains the following key components:
//!
//! * `path`: The base file path for the current log file.
//! * `file`: The current log file being written to.
//!
//! The `RollingLogger` struct provides methods to create a new logger,
//! write data to the log, flush the log, and rotate the log file based on
//! the specified strategy.

#![deny(missing_docs)]

mod log_file;
mod strategy;

pub use strategy::RotateStrategy;

use grafbase_workspace_hack as _;
use log_file::LogFile;
use std::{
    io::{self, Write},
    path::{Path, PathBuf},
};

/// A logger that rolls over based on a specified strategy (time duration or file size).
///
/// # Fields
///
/// * `path` - The base file path for the current log file.
/// * `file` - The current log file being written to.
#[derive(Debug)]
pub struct RollingLogger {
    path: PathBuf,
    file: LogFile,
}

impl RollingLogger {
    /// Creates a new `RollingLogger` with the given path and rotation strategy.
    ///
    /// # Arguments
    ///
    /// * `path` - A reference to the base file path for the current log file.
    /// * `rotate_strategy` - The strategy to use for rotating the log file.
    ///
    /// # Returns
    ///
    /// An `io::Result` which is `Ok` if the `RollingLogger` was successfully created,
    /// or an `io::Error` if there was a problem creating the log file.
    pub fn new(path: &Path, rotate_strategy: RotateStrategy) -> io::Result<Self> {
        let file = LogFile::new(path, rotate_strategy)?;

        Ok(Self {
            path: path.to_owned(),
            file,
        })
    }

    /// Flushes the current log file and rotates it according to the specified strategy.
    ///
    /// This method first flushes the current log file to ensure all pending data is written.
    /// It then renames the current log file by appending a timestamp to its name and creates
    /// a new log file to continue logging.
    ///
    /// # Returns
    ///
    /// An `io::Result<()>` which is `Ok` if the operation was successful, or an `io::Error`
    /// if there was a problem during the flush, rename, or creation of the new log file.
    fn flush_and_rotate(&mut self) -> io::Result<()> {
        self.flush()?;

        let mut path = self.path.as_os_str().to_os_string();
        path.push(format!(".{}", self.file.timestamp()));

        std::fs::rename(&self.path, &path)?;

        self.file = LogFile::new(&self.path, self.file.copy_new_rotate())?;

        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn set_rotate_start(&mut self, start: std::time::SystemTime) {
        self.file.set_rotate_start(start);
    }
}

impl Write for RollingLogger {
    /// Writes a buffer into the current log file. If the log file needs to be rotated
    /// according to the specified strategy, it flushes and rotates the log file before
    /// writing the buffer.
    ///
    /// # Arguments
    ///
    /// * `buf` - A byte slice that contains the data to be written to the log file.
    ///
    /// # Returns
    ///
    /// An `io::Result<usize>` which is `Ok` containing the number of bytes written, or
    /// an `io::Error` if there was a problem during the write operation.
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.file.needs_rotation() {
            self.flush_and_rotate()?;
        }

        self.file.write(buf)
    }

    /// Flushes the current log file, ensuring all buffered data is written to the file.
    ///
    /// This method is called automatically by the `RollingLogger` when needed, but can
    /// also be called manually if necessary.
    ///
    /// # Returns
    ///
    /// An `io::Result<()>` which is `Ok` if the flush operation was successful, or an
    /// `io::Error` if there was a problem during the flush operation.
    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::{RollingLogger, RotateStrategy};
    use std::{
        io::Write,
        time::{Duration, SystemTime},
    };
    use tempfile::TempDir;

    #[test]
    fn never_rotate() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("access.log");

        let mut logger = RollingLogger::new(&path, RotateStrategy::never()).unwrap();

        writeln!(&mut logger, "foo").unwrap();
        writeln!(&mut logger, "bar").unwrap();
        writeln!(&mut logger, "lol").unwrap();

        let data = std::fs::read_to_string(path).unwrap();

        insta::assert_snapshot!(&data, @r###"
        foo
        bar
        lol
        "###);
    }

    #[test]
    fn rotate_size() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("access.log");

        let mut logger = RollingLogger::new(&path, RotateStrategy::size(3)).unwrap();

        writeln!(&mut logger, "foo").unwrap();

        logger.set_rotate_start(SystemTime::UNIX_EPOCH + Duration::from_millis(1));
        writeln!(&mut logger, "bar").unwrap();

        logger.set_rotate_start(SystemTime::UNIX_EPOCH + Duration::from_millis(2));
        writeln!(&mut logger, "lol").unwrap();

        let data = std::fs::read_to_string(path).unwrap();

        insta::assert_snapshot!(&data, @r###"
        lol
        "###);

        let data = std::fs::read_to_string(dir.path().join("access.log.1")).unwrap();

        insta::assert_snapshot!(&data, @r###"
        foo
        "###);

        let data = std::fs::read_to_string(dir.path().join("access.log.2")).unwrap();

        insta::assert_snapshot!(&data, @r###"
        bar
        "###);
    }

    #[test]
    fn rotate_minutely() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("access.log");

        let mut logger = RollingLogger::new(&path, RotateStrategy::minutely()).unwrap();

        writeln!(&mut logger, "foo").unwrap();

        logger.set_rotate_start(SystemTime::now() - Duration::from_secs(59));
        writeln!(&mut logger, "bar").unwrap();

        logger.set_rotate_start(SystemTime::now() - Duration::from_secs(60));
        writeln!(&mut logger, "lol").unwrap();

        for file in dir.path().read_dir().unwrap() {
            let file = file.unwrap();
            let data = std::fs::read_to_string(file.path()).unwrap();

            if file.path().file_name().unwrap() == "access.log" {
                insta::assert_snapshot!(&data, @r###"
                lol
                "###);
            } else {
                insta::assert_snapshot!(&data, @r###"
                foo
                bar
                "###);
            }
        }
    }

    #[test]
    fn rotate_hourly() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("access.log");

        let mut logger = RollingLogger::new(&path, RotateStrategy::hourly()).unwrap();

        writeln!(&mut logger, "foo").unwrap();

        logger.set_rotate_start(SystemTime::now() - Duration::from_secs(60 * 60 - 1));
        writeln!(&mut logger, "bar").unwrap();

        logger.set_rotate_start(SystemTime::now() - Duration::from_secs(60 * 60));
        writeln!(&mut logger, "lol").unwrap();

        for file in dir.path().read_dir().unwrap() {
            let file = file.unwrap();
            let data = std::fs::read_to_string(file.path()).unwrap();

            if file.path().file_name().unwrap() == "access.log" {
                insta::assert_snapshot!(&data, @r###"
                lol
                "###);
            } else {
                insta::assert_snapshot!(&data, @r###"
                foo
                bar
                "###);
            }
        }
    }

    #[test]
    fn rotate_daily() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("access.log");

        let mut logger = RollingLogger::new(&path, RotateStrategy::daily()).unwrap();

        writeln!(&mut logger, "foo").unwrap();

        logger.set_rotate_start(SystemTime::now() - Duration::from_secs(60 * 60 * 24 - 1));
        writeln!(&mut logger, "bar").unwrap();

        logger.set_rotate_start(SystemTime::now() - Duration::from_secs(60 * 60 * 24));
        writeln!(&mut logger, "lol").unwrap();

        for file in dir.path().read_dir().unwrap() {
            let file = file.unwrap();
            let data = std::fs::read_to_string(file.path()).unwrap();

            if file.path().file_name().unwrap() == "access.log" {
                insta::assert_snapshot!(&data, @r###"
                lol
                "###);
            } else {
                insta::assert_snapshot!(&data, @r###"
                foo
                bar
                "###);
            }
        }
    }
}
