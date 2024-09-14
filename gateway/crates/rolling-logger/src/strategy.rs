use std::time::{Duration, SystemTime};

/// Represents the different strategies for log rotation.
#[derive(Debug, Clone, Copy)]
pub(crate) enum StrategyKind {
    /// Indicates that logs should never rotate.
    Never,
    /// Indicates that logs should rotate every minute.
    Minutely,
    /// Indicates that logs should rotate every hour.
    Hourly,
    /// Indicates that logs should rotate every day.
    Daily,
    /// Indicates that logs should rotate when the log file reaches the specified size in bytes.
    Size(u64),
}

/// A strategy that dictates when log rotation should occur.
#[derive(Debug, Clone, Copy)]
pub struct RotateStrategy {
    /// The kind of rotation strategy.
    kind: StrategyKind,
    /// The time when the rotation started.
    rotate_start: SystemTime,
}

impl RotateStrategy {
    /// Creates a new `RotateStrategy` that never rotates.
    ///
    /// # Returns
    ///
    /// A `RotateStrategy` configured to never rotate.
    pub fn never() -> Self {
        Self::new(StrategyKind::Never)
    }

    /// Creates a new `RotateStrategy` that rotates daily.
    ///
    /// # Returns
    ///
    /// A `RotateStrategy` configured to rotate daily.
    pub fn daily() -> Self {
        Self::new(StrategyKind::Daily)
    }

    /// Creates a new `RotateStrategy` that rotates hourly.
    ///
    /// # Returns
    ///
    /// A `RotateStrategy` configured to rotate hourly.
    pub fn hourly() -> Self {
        Self::new(StrategyKind::Hourly)
    }

    /// Creates a new `RotateStrategy` that rotates every minute.
    ///
    /// # Returns
    ///
    /// A `RotateStrategy` configured to rotate every minute.
    pub fn minutely() -> Self {
        Self::new(StrategyKind::Minutely)
    }

    /// Creates a new `RotateStrategy` that rotates when the log reaches the given size in bytes.
    ///
    /// This strategy will trigger a rotation when the log file exceeds the specified maximum size.
    /// The rotation occurs after the next log write that exceeds this size.
    ///
    /// # Arguments
    ///
    /// - `max_size`: The maximum size in bytes when rotation occurs.
    ///
    /// # Returns
    ///
    /// A `RotateStrategy` configured to rotate based on the specified size.
    pub fn size(max_size: u64) -> Self {
        Self::new(StrategyKind::Size(max_size))
    }

    /// Creates a new `RotateStrategy` with the specified kind of rotation.
    ///
    /// # Arguments
    ///
    /// - `kind`: The kind of rotation strategy to use.
    ///
    /// # Returns
    ///
    /// A `RotateStrategy` configured with the specified rotation kind.
    fn new(kind: StrategyKind) -> Self {
        Self {
            kind,
            rotate_start: SystemTime::now(),
        }
    }

    /// Returns the timestamp when the rotation started.
    ///
    /// This method returns a Unix timestamp (in milliseconds) representing
    /// the moment the logger started rotating logs. It's useful for calculating
    /// time intervals or differences between rotations.
    pub(crate) fn start_timestamp(self) -> u128 {
        self.rotate_start
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or_default()
    }

    /// Returns a copy of the current `RotateStrategy` with a new rotation start time.
    pub(crate) fn copy_new_start(self) -> Self {
        Self {
            kind: self.kind,
            rotate_start: SystemTime::now(),
        }
    }

    /// Returns the duration of time since the rotation started.
    ///
    /// This method calculates the elapsed time from when the logging rotation
    /// began until the current moment. The result is a `Duration` object that
    /// can be used to determine how long the logger has been active.
    ///
    /// # Returns
    ///
    /// A `Duration` representing the time since the rotation started.
    pub(crate) fn lifetime(self) -> Duration {
        SystemTime::now().duration_since(self.rotate_start).unwrap_or_default()
    }

    /// Returns the rotation strategy kind.
    ///
    /// This method retrieves the kind of rotation strategy currently in use,
    /// which dictates how and when log rotation occurs. The returned value
    /// can be one of the defined `StrategyKind` variants, such as `Never`,
    /// `Minutely`, `Hourly`, `Daily`, or `Size`.
    ///
    /// # Returns
    ///
    /// A `StrategyKind` indicating the type of rotation strategy.
    pub(crate) fn kind(self) -> StrategyKind {
        self.kind
    }

    #[cfg(test)]
    pub(crate) fn set_rotate_start(&mut self, start: SystemTime) {
        self.rotate_start = start;
    }
}
