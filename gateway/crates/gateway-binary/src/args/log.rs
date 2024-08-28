use std::fmt;

use clap::ValueEnum;

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub(crate) enum LogLevel {
    /// Completely disables logging.
    Off,
    /// Only errors.
    Error,
    /// Warnings and errors.
    Warn,
    /// Info, warning and error messages.
    #[default]
    Info,
    /// Debug, info, warning and error messages. Beware that debug messages will include sensitive
    /// information like request variables, responses, etc. Do not use it in production.
    Debug,
    /// Trace, debug, info, warning and error messages. Similar to debug, this will include
    /// sensitive information and should not be used in production.
    Trace,
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
pub(crate) enum LogStyle {
    /// Pretty printed logs, used as the default in the terminal
    Pretty,
    /// Standard text, used as the default when piping stdout to a file.
    Text,
    /// JSON objects
    Json,
}

impl Default for LogStyle {
    fn default() -> Self {
        let is_terminal = atty::is(atty::Stream::Stdout);
        if is_terminal {
            LogStyle::Pretty
        } else {
            LogStyle::Text
        }
    }
}

impl AsRef<str> for LogStyle {
    fn as_ref(&self) -> &str {
        match self {
            LogStyle::Pretty => "pretty",
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
