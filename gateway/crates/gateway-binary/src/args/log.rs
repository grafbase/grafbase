use std::fmt;

use clap::ValueEnum;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub(crate) enum LogLevel {
    /// Completely disables logging
    Off,
    /// Only errors from Grafbase libraries
    Error,
    /// Warnings and errors from Grafbase libraries
    Warn,
    /// Info, warning and error messages from Grafbase libraries
    Info,
    /// Debug, info, warning and error messages from all dependencies
    Debug,
    /// Trace, debug, info, warning and error messages from all dependencies
    Trace,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

impl LogLevel {
    pub(crate) fn as_filter_str(&self) -> &'static str {
        match self {
            LogLevel::Off => "off",
            LogLevel::Error => "grafbase=error,off",
            LogLevel::Warn => "grafbase=warn,off",
            LogLevel::Info => "grafbase=info,off",
            LogLevel::Debug => "grafbase=debug,off",
            LogLevel::Trace => "trace",
        }
    }
}

impl AsRef<str> for LogLevel {
    fn as_ref(&self) -> &str {
        match self {
            LogLevel::Off => "off",
            LogLevel::Error => "error",
            LogLevel::Warn => "warn",
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
            LogLevel::Trace => "trace",
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub(super) enum LogStyle {
    /// Standard text
    Text,
    /// JSON objects
    Json,
}

impl AsRef<str> for LogStyle {
    fn as_ref(&self) -> &str {
        match self {
            LogStyle::Text => "text",
            LogStyle::Json => "json",
        }
    }
}

impl fmt::Display for LogStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}
