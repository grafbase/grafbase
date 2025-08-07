use clap::ValueEnum;
use std::fmt;

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum LogStyle {
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
