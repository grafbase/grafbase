//! Logging utilities and abstractions.
//!
//! This module provides a few different loggers. The system logger is always enabled, and
//! the guest can call it by using the macros in the log module.
//!
//! The file logger is a special logger that writes logs to files with configurable rotation
//! policies. The user decides the serialization format for the logs.

pub use log;
mod file;
mod system;

pub use file::{FileLogger, LogRotation};
pub(crate) use system::HostLogger;
