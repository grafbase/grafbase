use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Copy)]
pub(crate) enum StrategyKind {
    Never,
    Minutely,
    Hourly,
    Daily,
    MaxSize(u64),
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
        Self::new(StrategyKind::MaxSize(max_size))
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
