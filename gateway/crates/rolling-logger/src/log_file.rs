use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

use crate::{strategy::StrategyKind, RotateStrategy};

const MINUTE_SECS: u64 = 60;
const HOUR_SECS: u64 = MINUTE_SECS * 60;
const DAY_SECS: u64 = HOUR_SECS * 24;

#[derive(Debug)]
pub(crate) struct LogFile {
    size: u64,
    file: File,
    rotate_strategy: RotateStrategy,
}

impl LogFile {
    /// Create a new file with accounting for size and the strategy for rotation.
    pub fn new(path: &Path, rotate_strategy: RotateStrategy) -> io::Result<Self> {
        let file = File::create(path)?;
        let size = fs::metadata(path).map_or(0, |m| m.len());

        Ok(Self {
            size,
            file,
            rotate_strategy,
        })
    }

    /// True, if the file has reached its limit and needs to be rotated.
    pub fn needs_rotation(&self) -> bool {
        match self.rotate_strategy.kind() {
            StrategyKind::Never => false,
            StrategyKind::Minutely => self.rotate_strategy.lifetime().as_secs() >= MINUTE_SECS,
            StrategyKind::Hourly => self.rotate_strategy.lifetime().as_secs() >= HOUR_SECS,
            StrategyKind::Daily => self.rotate_strategy.lifetime().as_secs() >= DAY_SECS,
            StrategyKind::Size(max_size) => self.size >= max_size,
        }
    }

    /// A timestamp given to the file when rotating.
    pub fn timestamp(&self) -> u128 {
        self.rotate_strategy.start_timestamp()
    }

    /// Creates a copy from current rotate strategy with a new start timestamp.
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
