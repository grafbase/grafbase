use std::{
    fmt::Write,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    path::{Path, PathBuf},
    time::Duration,
};

use crate::test::{
    GraphqlRequest, LogLevel,
    config::{CLI_BINARY_NAME, GATEWAY_BINARY_NAME, TestConfig},
    request::Body,
};

use anyhow::Context;
use grafbase_sdk_mock::{MockGraphQlServer, Subgraph};
use graphql_composition::{LoadedExtension, Subgraphs};
use itertools::Itertools;
use regex::Regex;
use tempfile::TempDir;
use url::Url;

/// A test runner that can start a gateway and execute GraphQL queries against it.
pub struct TestGateway {
    http_client: reqwest::Client,
    config: TestConfig,
    gateway_handle: Option<duct::Handle>,
    gateway_listen_address: SocketAddr,
    gateway_endpoint: Url,
    test_specific_temp_dir: TempDir,
    _mock_subgraphs: Vec<MockGraphQlServer>,
    federated_sdl: String,
}

#[derive(Debug, serde::Deserialize)]
struct ExtensionToml {
    extension: ExtensionDefinition,
}

#[derive(Debug, serde::Deserialize)]
struct ExtensionDefinition {
    name: String,
}

impl TestGateway {
    /// Creates a new test configuration builder.
    pub fn builder() -> TestGatewayBuilder {
        TestGatewayBuilder::new()
    }

    /// Creates a new GraphQL query builder with the given query.
    ///
    /// # Arguments
    ///
    /// * `query` - The GraphQL query string to execute
    ///
    /// # Returns
    ///
    /// A [`QueryBuilder`] that can be used to customize and execute the query
    pub fn query(&self, query: impl Into<Body>) -> GraphqlRequest {
        let builder = self.http_client.post(self.gateway_endpoint.clone());
        GraphqlRequest {
            builder,
            body: query.into(),
        }
    }

    /// Returns the federated schema as a string.
    pub fn federated_sdl(&self) -> &str {
        &self.federated_sdl
    }
}

#[derive(Debug, Default, Clone)]
/// Builder pattern to create a [`TestGateway`].
pub struct TestGatewayBuilder {
    gateway_path: Option<PathBuf>,
    cli_path: Option<PathBuf>,
    extension_path: Option<PathBuf>,
    toml_config: Option<String>,
    subgraphs: Vec<Subgraph>,
    enable_stdout: Option<bool>,
    enable_stderr: Option<bool>,
    enable_networking: Option<bool>,
    enable_environment_variables: Option<bool>,
    stream_stdout_stderr: Option<bool>,
    max_pool_size: Option<usize>,
    log_level: Option<LogLevel>,
}

impl TestGatewayBuilder {
    /// Creates a new [`TestConfigBuilder`] with default values.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Adds a subgraph to the test configuration.
    pub fn subgraph(mut self, subgraph: impl Into<Subgraph>) -> Self {
        self.subgraphs.push(subgraph.into());
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
    pub fn with_extension_path(mut self, extension_path: impl Into<PathBuf>) -> Self {
        self.extension_path = Some(extension_path.into().canonicalize().unwrap());
        self
    }

    /// Enables stdout output from the gateway and CLI. Useful for debugging errors in the gateway
    /// and in the extension.
    pub fn enable_stdout(mut self) -> Self {
        self.enable_stdout = Some(true);
        self
    }

    /// Enables stderr output from the gateway and CLI. Useful for debugging errors in the gateway
    /// and in the extension.
    pub fn enable_stderr(mut self) -> Self {
        self.enable_stderr = Some(true);
        self
    }

    /// Enables networking for the extension.
    pub fn enable_networking(mut self) -> Self {
        self.enable_networking = Some(true);
        self
    }

    /// Enables environment variables for the extension.
    pub fn enable_environment_variables(mut self) -> Self {
        self.enable_environment_variables = Some(true);
        self
    }

    /// Sets the maximum pool size for the extension.
    pub fn max_pool_size(mut self, size: usize) -> Self {
        self.max_pool_size = Some(size);
        self
    }

    /// Sets the log level for the gateway process output.
    pub fn log_level(mut self, level: impl Into<LogLevel>) -> Self {
        self.log_level = Some(level.into());
        self
    }

    /// Sets the TOML configuration for the gateway. The extension and subgraphs will be
    /// automatically added to the configuration.
    pub fn toml_config(mut self, cfg: impl ToString) -> Self {
        self.toml_config = Some(cfg.to_string());
        self
    }

    /// Stream stdout and stderr from the gateway & cli commands.
    /// Useful if you need to debug subscriptions for example. Not recommended in a CI for
    /// reporting clarity.
    pub fn stream_stdout_stderr(mut self) -> Self {
        self.stream_stdout_stderr = Some(true);
        self
    }

    /// Build the [`TestGateway`]
    pub async fn build(self) -> anyhow::Result<TestGateway> {
        let Self {
            gateway_path,
            cli_path,
            extension_path,
            enable_stdout,
            enable_stderr,
            subgraphs: mock_subgraphs,
            enable_networking,
            enable_environment_variables,
            max_pool_size,
            log_level,
            toml_config,
            stream_stdout_stderr,
        } = self;

        let gateway_path = match gateway_path {
            Some(path) => path,
            None => which::which(GATEWAY_BINARY_NAME).context("Could not fild grafbase-gateway binary in the PATH. Either install it or specify the gateway path in the test configuration.")?,
        };

        let cli_path = match cli_path {
            Some(path) => path,
            None => which::which(CLI_BINARY_NAME).context("Could not fild grafbase binary in the PATH. Either install it or specify the gateway path in the test configuration.")?,
        };

        let log_level = log_level.unwrap_or_default();

        TestGateway::setup(TestConfig {
            gateway_path,
            cli_path,
            toml_config: toml_config.unwrap_or_default(),
            extension_path,
            enable_stdout,
            enable_stderr,
            mock_subgraphs,
            enable_networking,
            enable_environment_variables,
            max_pool_size,
            log_level,
            stream_stdout_stderr,
        })
        .await
    }
}

#[allow(clippy::panic)]
impl TestGateway {
    async fn setup(mut config: TestConfig) -> anyhow::Result<Self> {
        let test_specific_temp_dir = tempfile::Builder::new().prefix("sdk-tests").tempdir()?;
        let gateway_listen_address = listen_address()?;
        let gateway_endpoint = Url::parse(&format!("http://{}/graphql", gateway_listen_address))?;

        let extension_toml_path = std::env::current_dir()?.join("extension.toml");
        let extension_toml = std::fs::read_to_string(&extension_toml_path)?;
        let extension_toml: ExtensionToml = toml::from_str(&extension_toml)?;
        let extension_name = extension_toml.extension.name;

        let mut mock_subgraphs = Vec::new();
        let mut subgraphs = Subgraphs::default();

        let extension_path = match config.extension_path {
            Some(ref path) => path.to_path_buf(),
            None => std::env::current_dir()?.join("build"),
        };

        let extension_url = url::Url::from_file_path(&extension_path).unwrap();
        subgraphs.ingest_loaded_extensions(std::iter::once(LoadedExtension::new(
            extension_url.to_string(),
            extension_name.clone(),
        )));

        let re = Regex::new(r#"@link\(\s*url\s*:\s*"(<self>)""#).unwrap();
        let rep = format!(r#"@link(url: "{extension_url}""#);
        for subgraph in config.mock_subgraphs.drain(..) {
            match subgraph {
                Subgraph::Graphql(subgraph) => {
                    let mock_graph = subgraph.start().await;
                    let sdl = re.replace_all(mock_graph.schema(), &rep);
                    subgraphs.ingest_str(sdl.as_ref(), mock_graph.name(), Some(mock_graph.url().as_str()))?;
                    mock_subgraphs.push(mock_graph);
                }
                Subgraph::Virtual(subgraph) => {
                    let sdl = re.replace_all(subgraph.schema(), &rep);
                    subgraphs.ingest_str(sdl.as_ref(), subgraph.name(), None)?;
                }
            }
        }

        let federated_graph = match graphql_composition::compose(&subgraphs)
            .warnings_are_fatal()
            .into_result()
        {
            Ok(graph) => graph,
            Err(diagnostics) => {
                return Err(anyhow::anyhow!(
                    "Failed to compose subgraphs:\n{}\n",
                    diagnostics
                        .iter_messages()
                        .format_with("\n", |msg, f| f(&format_args!("- {msg}")))
                ));
            }
        };
        let federated_sdl = graphql_composition::render_federated_sdl(&federated_graph)?;

        let mut this = Self {
            http_client: reqwest::Client::new(),
            config,
            gateway_handle: None,
            gateway_listen_address,
            gateway_endpoint,
            test_specific_temp_dir,
            _mock_subgraphs: mock_subgraphs,
            federated_sdl,
        };

        if this.config.extension_path.is_none() {
            this.build_extension(&extension_path)?;
        }

        this.start_servers(&extension_name, &extension_path)
            .await
            .map_err(|err| anyhow::anyhow!("Failed to start servers: {err}"))?;

        Ok(this)
    }

    async fn start_servers(&mut self, extension_name: &str, extension_path: &Path) -> anyhow::Result<()> {
        let extension_path = extension_path.display();
        let config_path = self.test_specific_temp_dir.path().join("grafbase.toml");
        let schema_path = self.test_specific_temp_dir.path().join("federated-schema.graphql");

        let config = {
            let max_pool_size = self.config.max_pool_size.unwrap_or(100);
            let mut config = indoc::formatdoc! {r#"
                [extensions.{extension_name}]
                path = "{extension_path}"
                max_pool_size = {max_pool_size}
            "#};
            if let Some(enabled) = self.config.enable_stderr {
                writeln!(config, "stderr = {enabled}").unwrap();
            }
            if let Some(enabled) = self.config.enable_stdout {
                writeln!(config, "stdout = {enabled}").unwrap();
            }
            if let Some(enabled) = self.config.enable_networking {
                writeln!(config, "networking = {enabled}").unwrap();
            }
            if let Some(enabled) = self.config.enable_environment_variables {
                writeln!(config, "environment_variables = {enabled}").unwrap();
            }
            config.push_str("\n\n");
            config.push_str(&self.config.toml_config);
            config
        };
        println!("{config}");

        std::fs::write(&config_path, config.as_bytes())
            .map_err(|err| anyhow::anyhow!("Failed to write config at {:?}: {err}", config_path))?;
        std::fs::write(&schema_path, self.federated_sdl.as_bytes())
            .map_err(|err| anyhow::anyhow!("Failed to write schema at {:?}: {err}", schema_path))?;

        let args = &[
            "--listen-address",
            &self.gateway_listen_address.to_string(),
            "--config",
            &config_path.to_string_lossy(),
            "--schema",
            &schema_path.to_string_lossy(),
            "--log",
            self.config.log_level.as_ref(),
        ];

        let gateway_handle = {
            let cmd = duct::cmd(&self.config.gateway_path, args);
            if self.config.stream_stdout_stderr.unwrap_or(false) {
                cmd
            } else {
                cmd.stdout_capture().stderr_capture()
            }
        }
        .unchecked()
        .stderr_to_stdout()
        .start()
        .map_err(|err| anyhow::anyhow!("Failed to start the gateway: {err}"))?;

        let mut i = 0;
        while !self.check_gateway_health().await? {
            // printing every second only
            if i % 10 == 0 {
                match gateway_handle.try_wait() {
                    Ok(Some(output)) => panic!(
                        "Gateway process exited unexpectedly: {}\n{}\n{}",
                        output.status,
                        String::from_utf8_lossy(&output.stdout),
                        String::from_utf8_lossy(&output.stderr)
                    ),
                    Ok(None) => (),
                    Err(err) => panic!("Error waiting for gateway process: {}", err),
                }
                println!("Waiting for gateway to be ready...");
            }
            i += 1;
            std::thread::sleep(Duration::from_millis(100));
        }

        self.gateway_handle = Some(gateway_handle);

        Ok(())
    }

    async fn check_gateway_health(&self) -> anyhow::Result<bool> {
        let url = self.gateway_endpoint.join("/health")?;

        let Ok(result) = self.http_client.get(url).send().await else {
            return Ok(false);
        };

        let result = result.error_for_status().is_ok();

        Ok(result)
    }

    fn build_extension(&mut self, extension_path: &Path) -> anyhow::Result<()> {
        let extension_path = extension_path.to_string_lossy();

        // Only one test can build the extension at a time. The others must
        // wait.
        let mut lock_file = fslock::LockFile::open(".build.lock")?;
        lock_file.lock()?;

        let args = &["extension", "build", "--debug", "--output-dir", &*extension_path];
        let output = {
            let cmd = duct::cmd(&self.config.cli_path, args);
            if self.config.stream_stdout_stderr.unwrap_or(false) {
                cmd
            } else {
                cmd.stdout_capture().stderr_capture()
            }
        }
        .unchecked()
        .stderr_to_stdout()
        .run()?;
        if !output.status.success() {
            panic!(
                "Failed to build extension: {}\n{}\n{}",
                output.status,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        lock_file.unlock()?;

        Ok(())
    }
}

pub(crate) fn free_port() -> anyhow::Result<u16> {
    const INITIAL_PORT: u16 = 14712;

    let test_dir = std::env::temp_dir().join("grafbase/sdk-tests");
    std::fs::create_dir_all(&test_dir)?;

    let lock_file_path = test_dir.join("port-number.lock");
    let port_number_file_path = test_dir.join("port-number.txt");

    let mut lock_file = fslock::LockFile::open(&lock_file_path)?;
    lock_file.lock()?;

    let port = if port_number_file_path.exists() {
        std::fs::read_to_string(&port_number_file_path)?.trim().parse::<u16>()? + 1
    } else {
        INITIAL_PORT
    };

    std::fs::write(&port_number_file_path, port.to_string())?;
    lock_file.unlock()?;

    Ok(port)
}

pub(crate) fn listen_address() -> anyhow::Result<SocketAddr> {
    let port = free_port()?;
    Ok(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port)))
}

impl Drop for TestGateway {
    fn drop(&mut self) {
        let Some(handle) = self.gateway_handle.take() else {
            return;
        };

        if let Err(err) = handle.kill() {
            eprintln!("Failed to kill grafbase-gateway: {}", err)
        }
    }
}
