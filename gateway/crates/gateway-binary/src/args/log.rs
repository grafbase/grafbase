use std::fmt;

use clap::ValueEnum;
use itertools::Itertools;
use tracing_subscriber::EnvFilter;

pub struct LogLevel<'a>(pub(super) &'a str);

impl std::ops::Deref for LogLevel<'_> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

// Verbose libraries in debug/trace mode which are rarely actually useful.
const LIBS: &[&str] = &["h2", "tower", "rustls", "hyper_util", "hyper", "reqwest"];

impl<'a> From<LogLevel<'a>> for EnvFilter {
    fn from(level: LogLevel<'a>) -> Self {
        EnvFilter::new(match level.0 {
            "off" | "error" | "warn" | "info" => level.0.to_string(),
            "debug" => format!(
                "debug,{}",
                LIBS.iter()
                    .format_with(",", |target, f| f(&format_args!("{target}=info")))
            ),
            "trace" => format!(
                "trace,{}",
                LIBS.iter()
                    .format_with(",", |target, f| f(&format_args!("{target}=info")))
            ),
            custom => custom.to_string(),
        })
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub(crate) enum LogStyle {
    /// Pretty printed logs, used as the default in the terminal when using debug logs
    Pretty,
    /// Standard text, used as the default when pretty isn't used.
    #[default]
    Text,
    /// JSON objects
    Json,
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
