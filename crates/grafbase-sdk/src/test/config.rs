use grafbase_sdk_mock::Subgraph;
use std::path::PathBuf;

pub(super) const GATEWAY_BINARY_NAME: &str = "grafbase-gateway";
pub(super) const CLI_BINARY_NAME: &str = "grafbase";

/// Log level for the test process output. Default value is `LogLevel::Error`.
#[derive(Debug, Clone, Default)]
pub enum LogLevel {
    /// Show all output from traces upwards.
    Trace,
    /// Show all output from debug upwards.
    Debug,
    /// Show all output from info upwards.
    #[default]
    Info,
    /// Show all output from warn upwards.
    Warn,
    /// Show only error messages.
    Error,
    /// Show all output from engine, debug upwards.
    EngineDebug,
    /// Extra verbose logs, show all output from engine, trace upwards.
    EngineTrace,
    /// Extra extra verbose logs
    WasiDebug,
    /// You know what you need
    Custom(String),
}

impl AsRef<str> for LogLevel {
    fn as_ref(&self) -> &str {
        match self {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
            LogLevel::EngineDebug => "engine=debug",
            LogLevel::EngineTrace => "engine=trace",
            LogLevel::WasiDebug => "wasi_component_loader=debug",
            LogLevel::Custom(level) => level.as_str(),
        }
    }
}

impl From<String> for LogLevel {
    fn from(level: String) -> Self {
        LogLevel::Custom(level)
    }
}

impl From<&str> for LogLevel {
    fn from(level: &str) -> Self {
        level.to_string().into()
    }
}

/// Configuration for test cases.
#[derive(Debug)]
pub(super) struct TestConfig {
    pub(super) gateway_path: PathBuf,
    pub(super) cli_path: PathBuf,
    pub(super) extension_path: Option<PathBuf>,
    pub(super) toml_config: String,
    pub(super) enable_stdout: Option<bool>,
    pub(super) enable_stderr: Option<bool>,
    pub(super) mock_subgraphs: Vec<Subgraph>,
    pub(super) enable_networking: Option<bool>,
    pub(super) enable_environment_variables: Option<bool>,
    pub(super) max_pool_size: Option<usize>,
    pub(super) log_level: LogLevel,
    pub(super) stream_stdout_stderr: Option<bool>,
}
