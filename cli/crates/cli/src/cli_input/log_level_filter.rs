use common::types::LogLevel;

#[derive(Default, Clone, Copy, Debug, PartialEq, PartialOrd, serde::Deserialize, clap::ValueEnum)]
#[clap(rename_all = "snake_case")]
pub enum LogLevelFilter {
    None,
    Error,
    Warn,
    #[default]
    Info,
    Debug,
}

impl LogLevelFilter {
    pub fn should_display(self, level: LogLevel) -> bool {
        Some(level)
            <= (match self {
                LogLevelFilter::None => None,
                LogLevelFilter::Error => Some(LogLevel::Error),
                LogLevelFilter::Warn => Some(LogLevel::Warn),
                LogLevelFilter::Info => Some(LogLevel::Info),
                LogLevelFilter::Debug => Some(LogLevel::Debug),
            })
    }
}

#[derive(Default, Clone, Copy)]
pub struct LogLevelFilters {
    pub functions: LogLevelFilter,
    pub graphql_operations: LogLevelFilter,
    pub fetch_requests: LogLevelFilter,
}
