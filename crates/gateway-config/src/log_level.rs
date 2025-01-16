use serde::{Deserialize, Deserializer, Serialize};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Off,
    Debug,
    Info,
    Warn,
    Error,
}

impl Serialize for LogLevel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Off => "off",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const VALUES: &[(&str, LogLevel)] = &[
            ("off", LogLevel::Off),
            ("debug", LogLevel::Debug),
            ("info", LogLevel::Info),
            ("warn", LogLevel::Warn),
            ("error", LogLevel::Error),
        ];

        VALUES
            .iter()
            .find(|(string, _log_level)| string.eq_ignore_ascii_case(s))
            .map(|(_, log_level)| *log_level)
            .ok_or_else(|| {
                format!(
                    r#""{s}" is not a valid log level (expected one of {})."#,
                    VALUES
                        .iter()
                        .map(|(string, _log_level)| *string)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })
    }
}

impl<'de> Deserialize<'de> for LogLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_level_from_string() {
        assert_eq!(LogLevel::from_str("off"), Ok(LogLevel::Off));
        assert_eq!(LogLevel::from_str("debug"), Ok(LogLevel::Debug));
        assert_eq!(LogLevel::from_str("info"), Ok(LogLevel::Info));
        assert_eq!(LogLevel::from_str("warn"), Ok(LogLevel::Warn));
        assert_eq!(LogLevel::from_str("error"), Ok(LogLevel::Error));
    }

    #[test]
    fn log_level_from_string_any_case() {
        assert_eq!(LogLevel::from_str("OFF"), Ok(LogLevel::Off));
        assert_eq!(LogLevel::from_str("dEbUg"), Ok(LogLevel::Debug));
        assert_eq!(LogLevel::from_str("InFo"), Ok(LogLevel::Info));
        assert_eq!(LogLevel::from_str("WARN"), Ok(LogLevel::Warn));
        assert_eq!(LogLevel::from_str("ERROR"), Ok(LogLevel::Error));
    }

    #[test]
    fn log_level_from_invalid_string() {
        assert_eq!(
            LogLevel::from_str("invalid"),
            Err(r#""invalid" is not a valid log level (expected one of off, debug, info, warn, error)."#.to_owned())
        );
    }
}
