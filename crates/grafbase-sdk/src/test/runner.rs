use std::{
    marker::PhantomData,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    path::PathBuf,
    time::Duration,
};

use graphql_composition::Subgraphs;
use tempdir::TempDir;
use url::Url;

use super::{mock::MockGraphQlServer, TestConfig};

/// A test runner that can start a gateway and execute GraphQL queries against it.
pub struct TestRunner {
    http_client: reqwest::Client,
    config: TestConfig,
    gateway_handle: Option<duct::Handle>,
    gateway_listen_address: SocketAddr,
    gateway_endpoint: Url,
    test_specific_temp_dir: TempDir,
    _mock_subgraphs: Vec<MockGraphQlServer>,
    federated_graph: String,
}

#[derive(Debug, serde::Deserialize)]
struct ExtensionToml {
    extension: ExtensionDefinition,
}

#[derive(Debug, serde::Deserialize)]
struct ExtensionDefinition {
    name: String,
}

impl TestRunner {
    /// Creates a new [`TestRunner`] with the given [`TestConfig`].
    pub fn new(mut config: TestConfig) -> anyhow::Result<Self> {
        let test_specific_temp_dir = TempDir::new("sdk-tests")?;
        let gateway_listen_address = listen_address()?;
        let gateway_endpoint = Url::parse(&format!("http://{}/graphql", gateway_listen_address))?;

        let mut mock_subgraphs = Vec::new();
        let mut subgraphs = Subgraphs::default();

        for subgraph in config.mock_subgraphs.drain(..) {
            subgraphs.ingest_str(subgraph.sdl(), subgraph.name(), subgraph.url().as_str())?;
            mock_subgraphs.push(subgraph.start());
        }

        let federated_graph = graphql_composition::compose(&subgraphs).into_result().unwrap();
        let federated_graph = graphql_federated_graph::render_federated_sdl(&federated_graph)?;

        Ok(Self {
            http_client: reqwest::Client::new(),
            config,
            gateway_handle: None,
            gateway_listen_address,
            gateway_endpoint,
            test_specific_temp_dir,
            _mock_subgraphs: mock_subgraphs,
            federated_graph,
        })
    }

    /// Starts the gateway process together with the configured subgraphs.
    ///
    /// This method:
    /// 1. Builds or uses the provided extension
    /// 2. Creates temporary config and schema files
    /// 3. Launches the gateway process with the appropriate arguments
    ///
    /// The gateway process will continue running until the [`TestRunner`] is dropped.
    ///
    /// # Errors
    ///
    /// Will return an error if:
    /// - Extension building fails
    /// - Config file creation fails
    /// - Schema file creation fails
    /// - Gateway process fails to start
    #[must_use]
    pub async fn start_servers(&mut self) -> anyhow::Result<()> {
        let extension_path = self.build_extension()?;
        let extension_path = extension_path.to_string_lossy();

        let extension_toml_path = std::env::current_dir()?.join("extension.toml");
        let extension_toml = std::fs::read_to_string(&extension_toml_path)?;
        let extension_toml: ExtensionToml = toml::from_str(&extension_toml)?;
        let extension_name = extension_toml.extension.name;

        let config_path = self.test_specific_temp_dir.path().join("grafbase.toml");
        let schema_path = self.test_specific_temp_dir.path().join("federated-schema.graphql");
        let config = &self.config.gateway_configuration;
        let enable_stdout = !self.config.enable_stderr;
        let enable_stderr = !self.config.enable_stderr;
        let enable_networking = self.config.enable_networking;
        let enable_environment_variables = self.config.enable_environment_variables;
        let max_pool_size = self.config.max_pool_size.unwrap_or(100);

        let config = indoc::formatdoc! {r#"
            [extensions.{extension_name}]
            path = "{extension_path}"
            stdout = {enable_stdout}
            stderr = {enable_stderr}
            networking = {enable_networking}
            environment_variables = {enable_environment_variables}
            max_pool_size = {max_pool_size}

            {config}
        "#};

        println!("{config}");

        std::fs::write(&config_path, config.as_bytes())?;
        std::fs::write(&schema_path, self.federated_graph.as_bytes())?;

        let args = &[
            "--listen-address",
            &self.gateway_listen_address.to_string(),
            "--config",
            &config_path.to_string_lossy(),
            "--schema",
            &schema_path.to_string_lossy(),
        ];

        let mut expr = duct::cmd(&self.config.gateway_path, args);

        if !self.config.enable_stderr {
            expr = expr.stderr_null();
        }

        if !self.config.enable_stdout {
            expr = expr.stdout_null();
        }

        self.gateway_handle = Some(expr.start()?);

        while !self.check_gateway_health().await? {
            std::thread::sleep(Duration::from_millis(100));
        }

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

    fn build_extension(&mut self) -> anyhow::Result<PathBuf> {
        if let Some(path) = self.config.extension_path.as_ref() {
            return Ok(path.canonicalize()?);
        }

        // Only one test can build the extension at a time. The others must
        // wait.
        let mut lock_file = fslock::LockFile::open(".build.lock")?;
        lock_file.lock()?;
        let args = &["extension", "build", "--debug"];
        let mut expr = duct::cmd(&self.config.cli_path, args);

        if !self.config.enable_stdout {
            expr = expr.stdout_null();
        }

        if !self.config.enable_stderr {
            expr = expr.stderr_null();
        }

        expr.run()?;
        lock_file.unlock()?;

        Ok(std::env::current_dir()?.join("build"))
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
    pub fn graphql_query<Response>(&self, query: impl Into<String>) -> QueryBuilder<Response> {
        let reqwest_builder = self
            .http_client
            .post(self.gateway_endpoint.clone())
            .header(http::header::ACCEPT, "application/json");

        QueryBuilder {
            query: query.into(),
            variables: None,
            phantom: PhantomData,
            reqwest_builder,
        }
    }

    /// Returns the federated schema as a string.
    pub fn federated_graph(&self) -> &str {
        &self.federated_graph
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

impl Drop for TestRunner {
    fn drop(&mut self) {
        let Some(handle) = self.gateway_handle.take() else {
            return;
        };

        if let Err(err) = handle.kill() {
            eprintln!("Failed to kill grafbase-gateway: {}", err)
        }
    }
}

#[derive(serde::Serialize)]
#[must_use]
/// A builder for constructing GraphQL queries with customizable parameters and headers.
pub struct QueryBuilder<Response> {
    // These two will be serialized into the request
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<serde_json::Value>,

    // These won't
    #[serde(skip)]
    phantom: PhantomData<fn() -> Response>,
    #[serde(skip)]
    reqwest_builder: reqwest::RequestBuilder,
}

impl<Response> QueryBuilder<Response> {
    /// Adds variables to the GraphQL query.
    ///
    /// # Arguments
    ///
    /// * `variables` - The variables to include with the query, serializable to JSON
    pub fn with_variables(mut self, variables: impl serde::Serialize) -> Self {
        self.variables = Some(serde_json::to_value(variables).unwrap());
        self
    }

    /// Adds a header to the GraphQL request.
    pub fn with_header(self, name: &str, value: &str) -> Self {
        let Self {
            phantom,
            query,
            mut reqwest_builder,
            variables,
        } = self;

        reqwest_builder = reqwest_builder.header(name, value);

        Self {
            query,
            variables,
            phantom,
            reqwest_builder,
        }
    }

    /// Sends the GraphQL query and returns the response.
    ///
    /// # Returns
    ///
    /// The deserialized response from the GraphQL server
    ///
    /// # Errors
    ///
    /// Will return an error if:
    /// - Request serialization fails
    /// - Network request fails
    /// - Response deserialization fails
    pub async fn send(self) -> anyhow::Result<Response>
    where
        Response: for<'de> serde::Deserialize<'de>,
    {
        let json = serde_json::to_value(&self)?;
        Ok(self.reqwest_builder.json(&json).send().await?.json().await?)
    }
}
