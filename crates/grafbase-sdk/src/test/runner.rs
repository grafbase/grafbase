use std::{
    future::IntoFuture,
    marker::PhantomData,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    path::Path,
    time::Duration,
};

use super::TestConfig;
use async_tungstenite::tungstenite::handshake::client::Request;
use futures_util::{stream::BoxStream, StreamExt};
use grafbase_sdk_mock::{MockGraphQlServer, MockSubgraph};
use graphql_composition::{LoadedExtension, Subgraphs};
use graphql_ws_client::graphql::GraphqlOperation;
use http::{
    header::{IntoHeaderName, SEC_WEBSOCKET_PROTOCOL},
    HeaderValue,
};
use serde::de::DeserializeOwned;
use tempfile::TempDir;
use tungstenite::client::IntoClientRequest;
use url::Url;

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
    pub async fn new(mut config: TestConfig) -> anyhow::Result<Self> {
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

        subgraphs.ingest_loaded_extensions(std::iter::once(LoadedExtension::new(
            format!("file://{}", extension_path.display()),
            extension_name.clone(),
        )));

        for subgraph in config.mock_subgraphs.drain(..) {
            match subgraph {
                MockSubgraph::Dynamic(subgraph) => {
                    let mock_graph = subgraph.start().await;
                    subgraphs.ingest_str(mock_graph.sdl(), mock_graph.name(), Some(mock_graph.url().as_str()))?;
                    mock_subgraphs.push(mock_graph);
                }
                MockSubgraph::ExtensionOnly(subgraph) => {
                    subgraphs.ingest_str(subgraph.sdl(), subgraph.name(), None)?;
                }
            }
        }

        let federated_graph = graphql_composition::compose(&subgraphs).into_result().unwrap();
        let federated_graph = graphql_federated_graph::render_federated_sdl(&federated_graph)?;

        let mut this = Self {
            http_client: reqwest::Client::new(),
            config,
            gateway_handle: None,
            gateway_listen_address,
            gateway_endpoint,
            test_specific_temp_dir,
            _mock_subgraphs: mock_subgraphs,
            federated_graph,
        };

        this.build_extension(&extension_path)?;
        this.start_servers(&extension_name, &extension_path).await?;

        Ok(this)
    }

    async fn start_servers(&mut self, extension_name: &str, extension_path: &Path) -> anyhow::Result<()> {
        let extension_path = extension_path.display();
        let config_path = self.test_specific_temp_dir.path().join("grafbase.toml");
        let schema_path = self.test_specific_temp_dir.path().join("federated-schema.graphql");
        let config = &self.config.gateway_configuration;
        let enable_stdout = self.config.enable_stdout;
        let enable_stderr = self.config.enable_stdout;
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
            "--log",
            self.config.log_level.as_ref(),
        ];

        let mut expr = duct::cmd(&self.config.gateway_path, args);

        if !dbg!(self.config.enable_stderr) {
            expr = expr.stderr_null();
        }

        if !self.config.enable_stdout {
            expr = expr.stdout_null();
        }

        self.gateway_handle = Some(expr.start()?);

        let mut i = 0;
        while !self.check_gateway_health().await? {
            // printing every second only
            if i % 10 == 0 {
                println!("Waiting for gateway to be ready...");
            }
            i += 1;
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

    fn build_extension(&mut self, extension_path: &Path) -> anyhow::Result<()> {
        let extension_path = extension_path.to_string_lossy();

        // Only one test can build the extension at a time. The others must
        // wait.
        let mut lock_file = fslock::LockFile::open(".build.lock")?;
        lock_file.lock()?;

        let args = &["extension", "build", "--debug", "--output-dir", &*extension_path];
        let mut expr = duct::cmd(&self.config.cli_path, args);

        if !self.config.enable_stdout {
            expr = expr.stdout_null();
        }

        if !self.config.enable_stderr {
            expr = expr.stderr_null();
        }

        expr.run()?;
        lock_file.unlock()?;

        Ok(())
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

    ///
    /// # Arguments
    ///
    /// * `query` - The GraphQL subscription query string to execute
    ///
    /// # Returns
    ///
    /// A [`SubscriptionBuilder`] that can be used to customize and execute the subscription
    pub fn graphql_subscription<Response>(
        &self,
        query: impl Into<String>,
    ) -> anyhow::Result<SubscriptionBuilder<Response>> {
        let mut url = self.gateway_endpoint.clone();

        url.set_path("/ws");
        url.set_scheme("ws").unwrap();

        let mut request_builder = url.as_ref().into_client_request()?;

        request_builder
            .headers_mut()
            .insert(SEC_WEBSOCKET_PROTOCOL, HeaderValue::from_static("graphql-transport-ws"));

        let operation = Operation {
            query: query.into(),
            variables: None,
            phantom: PhantomData,
        };

        Ok(SubscriptionBuilder {
            operation,
            request_builder,
        })
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

#[must_use]
/// A builder for constructing GraphQL queries with customizable parameters and headers.
pub struct SubscriptionBuilder<Response> {
    operation: Operation<Response>,
    request_builder: Request,
}

#[derive(serde::Serialize)]
struct Operation<Response> {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<serde_json::Value>,
    #[serde(skip)]
    phantom: PhantomData<fn() -> Response>,
}

impl<Response> GraphqlOperation for Operation<Response>
where
    Response: DeserializeOwned,
{
    type Response = Response;
    type Error = serde_json::Error;

    fn decode(&self, data: serde_json::Value) -> Result<Self::Response, Self::Error> {
        serde_json::from_value(data)
    }
}

impl<Response> SubscriptionBuilder<Response>
where
    Response: DeserializeOwned + 'static,
{
    /// Adds variables to the GraphQL subscription.
    ///
    /// # Arguments
    ///
    /// * `variables` - The variables to include with the subscription, serializable to JSON
    pub fn with_variables(mut self, variables: impl serde::Serialize) -> Self {
        self.operation.variables = Some(serde_json::to_value(variables).unwrap());
        self
    }

    /// Adds a header to the GraphQL request.
    ///
    /// # Arguments
    ///
    /// * `name` - The header name
    /// * `value` - The header value
    pub fn with_header<K>(mut self, name: K, value: HeaderValue) -> Self
    where
        K: IntoHeaderName,
    {
        self.request_builder.headers_mut().insert(name, value);
        self
    }

    /// Subscribes to the GraphQL subscription and returns a stream of responses.
    ///
    /// # Returns
    ///
    /// A pinned stream that yields the deserialized subscription responses
    ///
    /// # Errors
    ///
    /// Will return an error if:
    /// - WebSocket connection fails
    /// - GraphQL subscription initialization fails
    pub async fn subscribe(self) -> anyhow::Result<BoxStream<'static, Response>> {
        let (connection, _) = async_tungstenite::tokio::connect_async(self.request_builder).await?;
        let (client, actor) = graphql_ws_client::Client::build(connection).await?;

        tokio::spawn(actor.into_future());

        let stream = client
            .subscribe(self.operation)
            .await?
            .map(move |item| -> Response { item.unwrap() });

        Ok(Box::pin(stream))
    }
}
