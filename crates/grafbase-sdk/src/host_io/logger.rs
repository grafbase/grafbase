//! Logging utilities and abstractions.
//!
//! This module provides a few different loggers. The system logger is always enabled, and
//! the guest can call it by using the macros in the [log] module. The guest will pick the
//! log filter from the gateway log filter setting. If you want to define a separate filter
//! for extensions, you can do so by passing the specific level as `extension=level` to the
//! log filter.
//!
//! The [FileLogger] is a special logger that writes logs to files with configurable rotation
//! policies. The user decides the serialization format for the logs.

mod file;
mod system;

pub use file::{FileLogger, LogRotation};
pub(crate) use system::HostLogger;
