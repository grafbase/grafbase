use std::{
    collections::hash_map::Entry,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};

use crate::test::{
    GraphqlRequest, LogLevel,
    config::{
        CLI_BINARY_NAME, ExtensionConfig, ExtensionToml, GATEWAY_BINARY_NAME, GatewayToml, StructuredExtensionConfig,
    },
    request::{Body, IntrospectionRequest},
};

use anyhow::{Context, anyhow};
use grafbase_sdk_mock::{MockGraphQlServer, Subgraph};
use graphql_composition::{LoadedExtension, Subgraphs};
use itertools::Itertools;
use regex::Regex;
use tempfile::TempDir;
use url::Url;

/// A test runner that can start a gateway and execute GraphQL queries against it.
pub struct TestGateway {
    http_client: reqwest::Client,
    handle: duct::Handle,
    url: Url,
    federated_sdl: String,
    // Kept to drop them at the right time.
    #[allow(unused)]
    tmp_dir: TempDir,
    #[allow(unused)]
    mock_subgraphs: Vec<MockGraphQlServer>,
}

impl TestGateway {
    /// Creates a new test configuration builder.
    pub fn builder() -> TestGatewayBuilder {
        TestGatewayBuilder::new()
    }

    /// Full url of the GraphQL endpoint on the gateway.
    pub fn url(&self) -> &Url {
        &self.url
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
        let builder = self.http_client.post(self.url.clone());
        GraphqlRequest {
            builder,
            body: query.into(),
        }
    }

    /// Returns the federated schema as a string.
    pub fn federated_sdl(&self) -> &str {
        &self.federated_sdl
    }

    /// Execute a GraphQL introspection query to retrieve the API schema as a string.
    /// Beware that introspection must be explicitly enabled with in the TOML config:
    /// ```toml
    /// [graph]
    /// introspection = true
    /// ```
    pub fn introspect(&self) -> IntrospectionRequest {
        let operation = cynic_introspection::IntrospectionQuery::with_capabilities(
            cynic_introspection::SpecificationVersion::October2021.capabilities(),
        );
        IntrospectionRequest(self.query(Body {
            query: Some(operation.query),
            variables: None,
        }))
    }

    /// Checks if the gateway is healthy by sending a request to the `/health` endpoint.
    pub async fn health(&self) -> anyhow::Result<()> {
        let url = self.url.join("/health")?;
        let _ = self.http_client.get(url).send().await?.error_for_status()?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
/// Builder pattern to create a [`TestGateway`].
pub struct TestGatewayBuilder {
    gateway_path: Option<PathBuf>,
    cli_path: Option<PathBuf>,
    toml_config: Option<String>,
    subgraphs: Vec<Subgraph>,
    stream_stdout_stderr: Option<bool>,
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

    /// Sets the TOML configuration for the gateway. The extension and subgraphs will be
    /// automatically added to the configuration.
    pub fn toml_config(mut self, cfg: impl ToString) -> Self {
        self.toml_config = Some(cfg.to_string());
        self
    }

    /// Sets the log level for the gateway process output.
    pub fn log_level(mut self, level: impl Into<LogLevel>) -> Self {
        self.log_level = Some(level.into());
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
        println!("Building the gateway:");

        let gateway_path = match self.gateway_path {
            Some(path) => path,
            None => which::which(GATEWAY_BINARY_NAME).context("Could not fild grafbase-gateway binary in the PATH. Either install it or specify the gateway path in the test configuration.")?,
        };

        let cli_path = match self.cli_path {
            Some(path) => path,
            None => which::which(CLI_BINARY_NAME).context("Could not fild grafbase binary in the PATH. Either install it or specify the gateway path in the test configuration.")?,
        };

        let log_level = self.log_level.unwrap_or_default();

        let extension_path = std::env::current_dir()?;
        let extension_name =
            toml::from_str::<ExtensionToml>(&std::fs::read_to_string(extension_path.join("extension.toml"))?)?
                .extension
                .name;

        // Ensure current extension is built and up to date.
        {
            println!("* Building current extension.");
            let lock_path = extension_path.join(".build.lock");
            let mut lock_file = fslock::LockFile::open(&lock_path)?;
            lock_file.lock()?;

            let output = {
                let cmd = duct::cmd(&cli_path, &["extension", "build", "--debug"]).dir(&extension_path);
                if self.stream_stdout_stderr.unwrap_or(false) {
                    cmd
                } else {
                    cmd.stdout_capture().stderr_capture()
                }
            }
            .unchecked()
            .stderr_to_stdout()
            .run()?;

            if !output.status.success() {
                return Err(anyhow!(
                    "Failed to build extension: {}\n{}\n{}",
                    output.status,
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                ));
            }

            lock_file.unlock()?;
            anyhow::Ok(())
        }?;

        println!("* Preparing the grafbase.toml & schema.graphql files.");
        // Update grafbase TOML with current extension path.
        let mut toml_config: GatewayToml = toml::from_str(&self.toml_config.unwrap_or_default())?;
        match toml_config.extensions.entry(extension_name.clone()) {
            Entry::Occupied(mut entry) => match entry.get_mut() {
                ExtensionConfig::Version(_) => {
                    return Err(anyhow!(
                        "Current extension {extension_name} cannot be specified with a version"
                    ));
                }
                ExtensionConfig::Structured(config) => {
                    config
                        .path
                        .get_or_insert_with(|| extension_path.join("build").to_string_lossy().into_owned());
                }
            },
            Entry::Vacant(entry) => {
                entry.insert(ExtensionConfig::Structured(StructuredExtensionConfig {
                    path: Some(extension_path.join("build").to_string_lossy().into_owned()),
                    version: None,
                    rest: Default::default(),
                }));
            }
        }

        // Composition
        let (federated_sdl, mock_subgraphs) = {
            let extensions = toml_config
                .extensions
                .iter()
                .map(|(name, config)| match config {
                    ExtensionConfig::Version(version) => Ok(new_loaded_extension(
                        format!("https://extensions.grafbase.com/{extension_name}/{version}")
                            .parse()
                            .unwrap(),
                        name.clone(),
                    )),
                    ExtensionConfig::Structured(StructuredExtensionConfig {
                        path: None, version, ..
                    }) => Ok(new_loaded_extension(
                        format!(
                            "https://extensions.grafbase.com/{extension_name}/{}",
                            version
                                .as_ref()
                                .ok_or_else(|| anyhow!("Missing path or version for extension '{name}'"))?
                        )
                        .parse()
                        .unwrap(),
                        name.clone(),
                    )),
                    ExtensionConfig::Structured(StructuredExtensionConfig { path: Some(path), .. }) => {
                        let mut path = PathBuf::from_str(path.as_str())
                            .context(format!("Invalid path for extension {name}: {path}"))?;
                        if path.is_relative() {
                            path = extension_path.join(path);
                        }
                        anyhow::Ok(new_loaded_extension(
                            Url::from_file_path(&path).unwrap(),
                            name.to_owned(),
                        ))
                    }
                })
                .collect::<anyhow::Result<Vec<_>>>()?;

            compose(self.subgraphs, &extension_path, extensions).await
        }?;

        if toml_config.wasm.cache_path.is_none() {
            toml_config.wasm.cache_path = Some(extension_path.join("build").join("wasm-cache"));
        }

        // Build test dir
        let tmp_dir = tempfile::Builder::new().prefix("sdk-tests").tempdir()?;
        let config_path = tmp_dir.path().join("grafbase.toml");
        let schema_path = tmp_dir.path().join("schema.graphql");

        std::fs::write(&config_path, toml::to_string(&toml_config)?).context("Failed to write grafbase.toml")?;
        std::fs::write(&schema_path, &federated_sdl).context("Failed to write schema.graphql")?;

        // Install other extensions if necessary.
        if toml_config.extensions.len() > 1 {
            println!("* Installing other extensions.");
            let output = {
                let cmd = duct::cmd(&cli_path, &["extension", "install"]).dir(tmp_dir.path());
                if self.stream_stdout_stderr.unwrap_or(false) {
                    cmd
                } else {
                    cmd.stdout_capture().stderr_capture()
                }
            }
            .unchecked()
            .stderr_to_stdout()
            .run()?;

            if !output.status.success() {
                return Err(anyhow!(
                    "Failed to install extensions: {}\n{}\n{}",
                    output.status,
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }

        println!("* Starting the gateway.");
        let listen_address = new_listen_address()?;
        let url = Url::parse(&format!("http://{listen_address}/graphql")).unwrap();

        let handle = {
            let cmd = duct::cmd(
                &gateway_path,
                &[
                    "--listen-address",
                    &listen_address.to_string(),
                    "--config",
                    &config_path.to_string_lossy(),
                    "--schema",
                    &schema_path.to_string_lossy(),
                    "--log",
                    log_level.as_ref(),
                ],
            )
            .dir(tmp_dir.path());
            if self.stream_stdout_stderr.unwrap_or(false) {
                cmd
            } else {
                cmd.stdout_capture().stderr_capture()
            }
        }
        .unchecked()
        .stderr_to_stdout()
        .start()
        .map_err(|err| anyhow!("Failed to start the gateway: {err}"))?;

        let gateway = TestGateway {
            http_client: reqwest::Client::new(),
            handle,
            url,
            tmp_dir,
            mock_subgraphs,
            federated_sdl,
        };

        let mut i = 0;
        while gateway.health().await.is_err() {
            // printing every second only
            if i % 10 == 0 {
                match gateway.handle.try_wait() {
                    Ok(Some(output)) => {
                        return Err(anyhow!(
                            "Gateway process exited unexpectedly: {}\n{}\n{}",
                            output.status,
                            String::from_utf8_lossy(&output.stdout),
                            String::from_utf8_lossy(&output.stderr)
                        ));
                    }
                    Ok(None) => (),
                    Err(err) => return Err(anyhow!("Error waiting for gateway process: {err}")),
                }
                println!("Waiting for gateway to be ready...");
            }
            i += 1;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok(gateway)
    }
}

pub(crate) fn new_listen_address() -> anyhow::Result<SocketAddr> {
    let port = free_port()?;
    Ok(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port)))
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

async fn compose(
    subgraphs: impl IntoIterator<Item = Subgraph>,
    extension_path: &Path,
    extensions: impl IntoIterator<Item = LoadedExtension>,
) -> anyhow::Result<(String, Vec<MockGraphQlServer>)> {
    let mut mock_subgraphs = Vec::new();
    let mut composition_subgraphs = Subgraphs::default();

    composition_subgraphs.ingest_loaded_extensions(extensions);

    let extension_url = url::Url::from_file_path(extension_path.join("build")).unwrap();
    let re = Regex::new(r#"@link\(\s*url\s*:\s*"(<self>)""#).unwrap();
    let rep = format!(r#"@link(url: "{extension_url}""#);

    for subgraph in subgraphs {
        match subgraph {
            Subgraph::Graphql(subgraph) => {
                let mock_graph = subgraph.start().await;
                let sdl = re.replace_all(mock_graph.schema(), &rep);
                composition_subgraphs.ingest_str(sdl.as_ref(), mock_graph.name(), Some(mock_graph.url().as_str()))?;
                mock_subgraphs.push(mock_graph);
            }
            Subgraph::Virtual(subgraph) => {
                let sdl = re.replace_all(subgraph.schema(), &rep);
                composition_subgraphs.ingest_str(sdl.as_ref(), subgraph.name(), None)?;
            }
        }
    }

    let federated_graph = match graphql_composition::compose(&mut composition_subgraphs)
        .warnings_are_fatal()
        .into_result()
    {
        Ok(graph) => graph,
        Err(diagnostics) => {
            return Err(anyhow!(
                "Failed to compose subgraphs:\n{}\n",
                diagnostics
                    .iter_messages()
                    .format_with("\n", |msg, f| f(&format_args!("- {msg}")))
            ));
        }
    };
    let federated_sdl = graphql_composition::render_federated_sdl(&federated_graph)?;

    Ok((federated_sdl, mock_subgraphs))
}

impl Drop for TestGateway {
    fn drop(&mut self) {
        if let Err(err) = self.handle.kill() {
            eprintln!("Failed to kill grafbase-gateway: {err}")
        }
    }
}

fn new_loaded_extension(url: Url, name: String) -> LoadedExtension {
    LoadedExtension {
        link_url: url.to_string(),
        url,
        name,
    }
}
