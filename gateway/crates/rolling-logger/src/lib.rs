//! # Simple rolling logger
//!
//! Provides a logger with rotation. Rotates either based on time duration, or file size.
//! The currently written log is the given base file name. Rotated logs get the start timestamp
//! added to the suffix. E.g. if the logging started at timestamp 1, after the first rotation the
//! previous log filename is log_file.1.

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

/// A simple rolling logger.
#[derive(Debug)]
pub struct RollingLogger {
    path: PathBuf,
    file: LogFile,
}

impl RollingLogger {
    /// Creates a new rolling logger. The path should point to a file.
    pub fn new(path: &Path, rotate_strategy: RotateStrategy) -> io::Result<Self> {
        let file = LogFile::new(path, rotate_strategy)?;

        Ok(Self {
            path: path.to_owned(),
            file,
        })
    }

    /// Flush all buffered data to the file, move the file to an archive with a timestamp and start
    /// a new empty file.
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
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.file.needs_rotation() {
            self.flush_and_rotate()?;
        }

        self.file.write(buf)
    }

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
