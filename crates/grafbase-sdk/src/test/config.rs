use anyhow::Context;
use grafbase_sdk_mock::MockSubgraph;
use std::path::PathBuf;

const GATEWAY_BINARY_NAME: &str = "grafbase-gateway";
const CLI_BINARY_NAME: &str = "grafbase";

/// Log level for the test process output. Default value is `LogLevel::Error`.
#[derive(Debug, Clone, Copy, Default)]
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
}

impl AsRef<str> for LogLevel {
    fn as_ref(&self) -> &str {
        match self {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        }
    }
}

/// Configuration for test cases.
#[derive(Debug)]
pub struct TestConfig {
    pub(super) gateway_path: PathBuf,
    pub(super) cli_path: PathBuf,
    pub(super) extension_path: Option<PathBuf>,
    pub(super) gateway_configuration: String,
    pub(super) enable_stdout: bool,
    pub(super) enable_stderr: bool,
    pub(super) mock_subgraphs: Vec<MockSubgraph>,
    pub(super) enable_networking: bool,
    pub(super) enable_environment_variables: bool,
    pub(super) max_pool_size: Option<usize>,
    pub(super) log_level: LogLevel,
}

#[derive(Debug, Default)]
/// Builder pattern to create a [`TestConfig`].
pub struct TestConfigBuilder {
    gateway_path: Option<PathBuf>,
    cli_path: Option<PathBuf>,
    extension_path: Option<PathBuf>,
    mock_subgraphs: Vec<MockSubgraph>,
    enable_stdout: bool,
    enable_stderr: bool,
    enable_networking: bool,
    enable_environment_variables: bool,
    max_pool_size: Option<usize>,
    log_level: Option<LogLevel>,
}

impl TestConfigBuilder {
    /// Creates a new [`TestConfigBuilder`] with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a dynamic subgraph to the test configuration.
    pub fn with_subgraph(mut self, subgraph: impl Into<MockSubgraph>) -> Self {
        self.mock_subgraphs.push(subgraph.into());
        self
    }

    /// Specifies a custom path to the gateway binary. If not defined, the binary will be searched in the PATH.
    pub fn with_gateway(mut self, gateway_path: impl Into<PathBuf>) -> Self {
        self.gateway_path = Some(gateway_path.into());
        self
    }

    /// Specifies a custom path to the CLI binary. If not defined, the binary will be searched in the PATH.
    pub fn with_cli(mut self, cli_path: impl Into<PathBuf>) -> Self {
        self.cli_path = Some(cli_path.into());
        self
    }

    /// Specifies a path to a pre-built extension. If not defined, the extension will be built.
    pub fn with_extension(mut self, extension_path: impl Into<PathBuf>) -> Self {
        self.extension_path = Some(extension_path.into());
        self
    }

    /// Enables stdout output from the gateway and CLI. Useful for debugging errors in the gateway
    /// and in the extension.
    pub fn enable_stdout(mut self) -> Self {
        self.enable_stdout = true;
        self
    }

    /// Enables stderr output from the gateway and CLI. Useful for debugging errors in the gateway
    /// and in the extension.
    pub fn enable_stderr(mut self) -> Self {
        self.enable_stderr = true;
        self
    }

    /// Enables networking for the extension.
    pub fn enable_networking(mut self) -> Self {
        self.enable_networking = true;
        self
    }

    /// Enables environment variables for the extension.
    pub fn enable_environment_variables(mut self) -> Self {
        self.enable_environment_variables = true;
        self
    }

    /// Sets the maximum pool size for the extension.
    pub fn max_pool_size(mut self, size: usize) -> Self {
        self.max_pool_size = Some(size);
        self
    }

    /// Sets the log level for the gateway process output.
    pub fn log_level(mut self, level: LogLevel) -> Self {
        self.log_level = Some(level);
        self
    }

    /// Builds the [`TestConfig`] with the given gateway configuration and federated graph schema.
    pub fn build(self, gateway_configuration: impl ToString) -> anyhow::Result<TestConfig> {
        let Self {
            gateway_path,
            cli_path,
            extension_path,
            enable_stdout,
            enable_stderr,
            mock_subgraphs,
            enable_networking,
            enable_environment_variables,
            max_pool_size,
            log_level,
        } = self;

        let gateway_path = match gateway_path {
            Some(path) => path,
            None => which::which(GATEWAY_BINARY_NAME).context("Could not fild grafbase-gateway binary in the PATH. Either install it or specify the gateway path in the test configuration.")?,
        };

        let cli_path = match cli_path {
            Some(path) => path,
            None => which::which(CLI_BINARY_NAME).context("Could not fild grafbase binary in the PATH. Either install it or specify the gateway path in the test configuration.")?,
        };

        let gateway_configuration = gateway_configuration.to_string();
        let log_level = log_level.unwrap_or_default();

        Ok(TestConfig {
            gateway_path,
            cli_path,
            gateway_configuration,
            extension_path,
            enable_stdout,
            enable_stderr,
            mock_subgraphs,
            enable_networking,
            enable_environment_variables,
            max_pool_size,
            log_level,
        })
    }
}
