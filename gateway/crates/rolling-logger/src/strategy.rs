use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Copy)]
pub(crate) enum StrategyKind {
    /// Never rotate.
    Never,
    /// Rotates after a minute the logger has started.
    Minutely,
    /// Rotates after an hour the logger has started.
    Hourly,
    /// Rotates after a day the logger has started.
    Daily,
    /// Rotates when the log reaches the given size in bytes.
    /// We write first and if the next write is over, we rotate.
    Size(u64),
}

/// Defines a strategy when a log file gets rotated.
#[derive(Debug, Clone, Copy)]
pub struct RotateStrategy {
    kind: StrategyKind,
    rotate_start: SystemTime,
}

impl RotateStrategy {
    /// Never rotate.
    pub fn never() -> Self {
        Self::new(StrategyKind::Never)
    }

    /// Rotates every 24 hours from start.
    pub fn daily() -> Self {
        Self::new(StrategyKind::Daily)
    }

    /// Rotates every hour from start.
    pub fn hourly() -> Self {
        Self::new(StrategyKind::Hourly)
    }

    /// Rotates every minute from start.
    pub fn minutely() -> Self {
        Self::new(StrategyKind::Minutely)
    }

    /// Rotates when the file is either the given size or over.
    pub fn size(max_size: u64) -> Self {
        Self::new(StrategyKind::Size(max_size))
    }

    fn new(kind: StrategyKind) -> Self {
        Self {
            kind,
            rotate_start: SystemTime::now(),
        }
    }

    pub(crate) fn start_timestamp(self) -> u128 {
        self.rotate_start
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or_default()
    }

    pub(crate) fn copy_new_start(self) -> Self {
        Self {
            kind: self.kind,
            rotate_start: SystemTime::now(),
        }
    }

    pub(crate) fn lifetime(self) -> Duration {
        SystemTime::now().duration_since(self.rotate_start).unwrap_or_default()
    }

    pub(crate) fn kind(self) -> StrategyKind {
        self.kind
    }

    #[cfg(test)]
    pub(crate) fn set_rotate_start(&mut self, start: SystemTime) {
        self.rotate_start = start;
    }
}
