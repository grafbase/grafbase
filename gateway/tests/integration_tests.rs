mod access_logs;
mod entity_caching;
mod mocks;
mod telemetry;

use std::{
    borrow::Cow,
    env, fs,
    marker::PhantomData,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    panic::{catch_unwind, AssertUnwindSafe},
    path::{self, Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
    time::{Duration, SystemTime},
};

use crate::mocks::gdn::GdnResponseMock;
use duct::{cmd, Handle};
use futures_util::future::BoxFuture;
use futures_util::{Future, FutureExt};
use http::{HeaderMap, StatusCode};
use indoc::indoc;
use tempfile::tempdir;
use tokio::runtime::Runtime;
use tokio::time::Instant;
use wiremock::{
    matchers::{header, method, path},
    Mock, ResponseTemplate,
};

const ACCESS_TOKEN: &str = "test";

#[derive(serde::Serialize)]
struct BatchQuery {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<serde_json::Value>,
}

#[must_use]
pub struct GqlBatchBuilder<Response> {
    queries: Vec<BatchQuery>,
    phantom: PhantomData<fn() -> Response>,
    reqwest_builder: reqwest::RequestBuilder,
    bearer: Option<String>,
}

impl<Response> GqlBatchBuilder<Response> {
    pub async fn send(self) -> Response
    where
        Response: for<'de> serde::de::Deserialize<'de>,
    {
        let json = serde_json::to_value(&self.queries).expect("to be able to serialize gql request");

        if let Some(bearer) = self.bearer {
            self.reqwest_builder.header("authorization", bearer)
        } else {
            self.reqwest_builder
        }
        .json(&json)
        .send()
        .await
        .unwrap()
        .json::<Response>()
        .await
        .unwrap()
    }
}

#[derive(serde::Serialize)]
#[must_use]
pub struct GqlRequestBuilder<Response> {
    // These two will be serialized into the request
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<serde_json::Value>,

    // These won't
    #[serde(skip)]
    phantom: PhantomData<fn() -> Response>,
    #[serde(skip)]
    reqwest_builder: reqwest::RequestBuilder,
    #[serde(skip)]
    bearer: Option<String>,
}

impl<Response> GqlRequestBuilder<Response> {
    pub fn variables(mut self, variables: impl serde::Serialize) -> Self {
        self.variables = Some(serde_json::to_value(variables).expect("to be able to serialize variables"));
        self
    }

    pub fn bearer(mut self, token: &str) -> Self {
        self.bearer = Some(format!("Bearer {token}"));
        self
    }

    pub fn header(self, name: &str, value: &str) -> Self {
        let Self {
            bearer,
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
            bearer,
        }
    }

    pub async fn send(self) -> Response
    where
        Response: for<'de> serde::de::Deserialize<'de>,
    {
        let json = serde_json::to_value(&self).expect("to be able to serialize gql request");

        if let Some(bearer) = self.bearer {
            self.reqwest_builder.header("authorization", bearer)
        } else {
            self.reqwest_builder
        }
        .json(&json)
        .send()
        .await
        .unwrap()
        .json::<Response>()
        .await
        .unwrap()
    }

    pub async fn request(self) -> reqwest::Response {
        let json = serde_json::to_value(&self).expect("to be able to serialize gql request");
        self.reqwest_builder.json(&json).send().await.unwrap()
    }
}

pub struct Client {
    endpoint: String,
    client: reqwest::Client,
    headers: HeaderMap,
    commands: CommandHandles,
    schema_path: Option<PathBuf>,
}

impl Client {
    pub fn new(endpoint: String, commands: CommandHandles, schema_path: Option<PathBuf>) -> Self {
        Self {
            endpoint,
            headers: HeaderMap::new(),
            client: reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(1))
                .build()
                .unwrap(),
            commands,
            schema_path,
        }
    }

    pub fn schema_path(&self) -> Option<&Path> {
        self.schema_path.as_deref()
    }

    pub fn with_header(mut self, key: &'static str, value: impl AsRef<str>) -> Self {
        self.headers.insert(key, value.as_ref().parse().unwrap());
        self
    }

    pub async fn poll_endpoint(&self, timeout_secs: u64, interval_millis: u64) {
        let start = SystemTime::now();

        loop {
            let valid_response = self
                .client
                .head(&self.endpoint)
                .send()
                .await
                .is_ok_and(|response| response.status() != StatusCode::SERVICE_UNAVAILABLE);

            if valid_response {
                break;
            }

            assert!(start.elapsed().unwrap().as_secs() < timeout_secs, "timeout");

            tokio::time::sleep(Duration::from_millis(interval_millis)).await;
        }
    }

    pub fn kill_handles(&self) {
        self.commands.kill_all()
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn gql<Response>(&self, query: impl Into<String>) -> GqlRequestBuilder<Response>
    where
        Response: for<'de> serde::de::Deserialize<'de>,
    {
        let reqwest_builder = self.client.post(&self.endpoint).headers(self.headers.clone());

        GqlRequestBuilder {
            query: query.into(),
            variables: None,
            phantom: PhantomData,
            reqwest_builder: reqwest_builder.header(http::header::ACCEPT, "application/json"),
            bearer: None,
        }
    }

    pub fn gql_batch<Response, T>(&self, queries: impl IntoIterator<Item = T>) -> GqlBatchBuilder<Response>
    where
        Response: for<'de> serde::de::Deserialize<'de>,
        T: Into<String>,
    {
        let reqwest_builder = self.client.post(&self.endpoint).headers(self.headers.clone());

        let queries = queries
            .into_iter()
            .map(|query| BatchQuery {
                query: query.into(),
                variables: None,
            })
            .collect();

        GqlBatchBuilder {
            queries,
            phantom: PhantomData,
            reqwest_builder: reqwest_builder.header(http::header::ACCEPT, "application/json"),
            bearer: None,
        }
    }

    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }
}

#[derive(Clone)]
pub struct CommandHandles(Arc<Mutex<Vec<Handle>>>);

impl CommandHandles {
    pub fn new() -> Self {
        CommandHandles(Arc::new(Mutex::new(vec![])))
    }

    pub fn push(&mut self, handle: Handle) {
        self.0.lock().unwrap().push(handle);
    }

    pub fn still_running(&self) -> bool {
        self.0
            .lock()
            .unwrap()
            .iter()
            .all(|handle| handle.try_wait().unwrap().is_none())
    }

    pub fn kill_all(&self) {
        for command in self.0.lock().unwrap().iter() {
            command.kill().unwrap();
        }
    }
}

impl Default for CommandHandles {
    fn default() -> Self {
        Self::new()
    }
}

pub fn cargo_bin<S: AsRef<str>>(name: S) -> path::PathBuf {
    cargo_bin_str(name.as_ref())
}

fn target_dir() -> path::PathBuf {
    env::current_exe()
        .ok()
        .map(|mut path| {
            path.pop();
            if path.ends_with("deps") {
                path.pop();
            }
            path
        })
        .unwrap()
}

fn cargo_bin_str(name: &str) -> path::PathBuf {
    let env_var = format!("CARGO_BIN_EXE_{name}");
    std::env::var_os(env_var).map_or_else(
        || target_dir().join(format!("{name}{}", env::consts::EXE_SUFFIX)),
        std::convert::Into::into,
    )
}

fn runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
    })
}

fn get_free_port() -> u16 {
    const INITIAL_PORT: u16 = 14712;

    let test_state_directory_path = std::env::temp_dir().join("grafbase/cli-tests");
    std::fs::create_dir_all(&test_state_directory_path).unwrap();
    let lock_file_path = test_state_directory_path.join("port-number.lock");
    let port_number_file_path = test_state_directory_path.join("port-number.txt");
    let mut lock_file = fslock::LockFile::open(&lock_file_path).unwrap();
    lock_file.lock().unwrap();
    let port_number = if port_number_file_path.exists() {
        std::fs::read_to_string(&port_number_file_path)
            .unwrap()
            .trim()
            .parse::<u16>()
            .unwrap()
            + 1
    } else {
        INITIAL_PORT
    };
    std::fs::write(&port_number_file_path, port_number.to_string()).unwrap();
    lock_file.unlock().unwrap();
    port_number
}

fn listen_address() -> SocketAddr {
    let port = get_free_port();
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port))
}

struct ConfigContent<'a>(Option<Cow<'a, str>>);

impl<'a> From<&'a str> for ConfigContent<'a> {
    fn from(s: &'a str) -> Self {
        ConfigContent(Some(Cow::Borrowed(s)))
    }
}

impl<'a> From<&'a String> for ConfigContent<'a> {
    fn from(s: &'a String) -> Self {
        ConfigContent(Some(Cow::Borrowed(s)))
    }
}

impl From<String> for ConfigContent<'static> {
    fn from(s: String) -> Self {
        ConfigContent(Some(Cow::Owned(s)))
    }
}

struct GatewayBuilder<'a> {
    toml_config: ConfigContent<'a>,
    schema: &'a str,
    log_level: Option<String>,
    client_url_path: Option<&'a str>,
    client_headers: Option<&'static [(&'static str, &'static str)]>,
}

impl<'a> GatewayBuilder<'a> {
    fn new(schema: &'a str) -> Self {
        Self {
            toml_config: ConfigContent(None),
            schema,
            log_level: None,
            client_url_path: None,
            client_headers: None,
        }
    }

    fn with_log_level(mut self, level: &str) -> Self {
        self.log_level = Some(level.to_string());
        self
    }

    fn run<F>(self, test: impl FnOnce(Arc<Client>) -> F)
    where
        F: Future<Output = ()>,
    {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("grafbase.toml");

        let schema_path = temp_dir.path().join("schema.graphql");
        fs::write(&schema_path, self.schema).unwrap();

        let addr = listen_address();
        let mut args = vec![
            "--listen-address".to_string(),
            addr.to_string(),
            "--schema".to_string(),
            schema_path.to_str().unwrap().to_string(),
        ];

        if let Some(config) = self.toml_config.0 {
            fs::write(&config_path, config.as_ref()).unwrap();
            args.push("--config".to_string());
            args.push(config_path.to_str().unwrap().to_string());
        }

        if let Some(level) = self.log_level {
            args.push("--log".to_string());
            args.push(level);
        }

        let command = cmd(cargo_bin("grafbase-gateway"), &args).stdout_null().stderr_null();

        let endpoint = match self.client_url_path {
            Some(path) => format!("http://{addr}/{path}"),
            None => format!("http://{addr}/graphql"),
        };

        let mut commands = CommandHandles::new();
        commands.push(command.start().unwrap());

        let mut client = Client::new(endpoint, commands, Some(schema_path));

        if let Some(headers) = self.client_headers {
            for header in headers {
                client = client.with_header(header.0, header.1);
            }
        }

        let client = Arc::new(client);

        let res = catch_unwind(AssertUnwindSafe(|| {
            runtime().block_on(async {
                client.poll_endpoint(30, 300).await;
                test(client.clone()).await
            })
        }));

        client.kill_handles();

        if let Err(err) = res {
            std::panic::resume_unwind(err);
        }
    }
}

fn with_static_server<'a, F, T>(
    config: impl Into<ConfigContent<'a>>,
    schema: &str,
    path: Option<&str>,
    headers: Option<&'static [(&'static str, &'static str)]>,
    test: T,
) where
    T: FnOnce(Arc<Client>) -> F,
    F: Future<Output = ()>,
{
    GatewayBuilder {
        toml_config: config.into(),
        schema,
        log_level: None,
        client_url_path: path,
        client_headers: headers,
    }
    .run(test)
}

fn with_hybrid_server<F, T>(config: &str, graph_ref: &str, sdl: &str, test: T)
where
    T: FnOnce(Arc<Client>, GdnResponseMock, SocketAddr) -> F,
    F: Future<Output = ()>,
{
    let temp_dir = tempdir().unwrap();

    let config_path = temp_dir.path().join("grafbase.toml");
    fs::write(&config_path, config).unwrap();

    let addr = listen_address();

    let gdn_response = GdnResponseMock::mock(sdl);

    let res = runtime().block_on(async {
        let response = ResponseTemplate::new(200).set_body_string(gdn_response.as_json().to_string());
        let server = wiremock::MockServer::start().await;

        Mock::given(method("GET"))
            .and(path(format!("/graphs/{graph_ref}/current")))
            .and(header("Authorization", format!("Bearer {ACCESS_TOKEN}")))
            .respond_with(response)
            .mount(&server)
            .await;

        let command = cmd!(
            cargo_bin("grafbase-gateway"),
            "--listen-address",
            &addr.to_string(),
            "--config",
            &config_path.to_str().unwrap(),
            "--graph-ref",
            graph_ref,
        )
        .stdout_null()
        .stderr_null()
        .env("GRAFBASE_GDN_URL", format!("http://{}", server.address()))
        .env("GRAFBASE_ACCESS_TOKEN", ACCESS_TOKEN);

        let mut commands = CommandHandles::new();
        commands.push(command.start().unwrap());

        let client = Arc::new(Client::new(format!("http://{addr}/graphql"), commands, None));

        client.poll_endpoint(30, 300).await;

        let res = AssertUnwindSafe(test(client.clone(), gdn_response, *server.address()))
            .catch_unwind()
            .await;

        client.kill_handles();

        res
    });

    res.unwrap();
}

fn load_schema(name: &str) -> String {
    let path = format!("./tests/schemas/{name}.graphql");
    fs::read_to_string(path).unwrap()
}

async fn introspect(url: &str) -> String {
    grafbase_graphql_introspection::introspect(url, &[("x-api-key", "")])
        .await
        .unwrap_or_default()
}

pub fn clickhouse_client() -> &'static ::clickhouse::Client {
    static CLIENT: OnceLock<::clickhouse::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        ::clickhouse::Client::default()
            .with_url("http://localhost:8124")
            .with_user("default")
            .with_database("otel")
    })
}

#[ctor::ctor]
fn setup_rustls() {
    rustls::crypto::ring::default_provider().install_default().unwrap();
}

// This failed when using `format_with()` inside tracing with opentelemetry. Somehow it gets called
// multiple times which isn't supported and panics...
#[test]
fn trace_log_level() {
    let schema = load_schema("big");

    let query = indoc! {r#"
        query Me {
          me {
            id
          }
        }
    "#};

    GatewayBuilder::new(&schema)
        .with_log_level("trace")
        .run(|client| async move {
            let result: serde_json::Value = client.gql(query).send().await;
            let result = serde_json::to_string_pretty(&result).unwrap();

            insta::assert_snapshot!(&result, @r###"
            {
              "data": null,
              "errors": [
                {
                  "message": "Request to subgraph 'accounts' failed with: error sending request",
                  "path": [
                    "me"
                  ],
                  "extensions": {
                    "code": "SUBGRAPH_REQUEST_ERROR"
                  }
                }
              ]
            }
            "###);
        })
}

#[test]
fn no_config() {
    let schema = load_schema("big");

    let query = indoc! {r#"
        query Me {
          me {
            id
          }
        }
    "#};

    with_static_server(ConfigContent(None), &schema, None, None, |client| async move {
        let result: serde_json::Value = client.gql(query).send().await;
        let result = serde_json::to_string_pretty(&result).unwrap();

        insta::assert_snapshot!(&result, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Request to subgraph 'accounts' failed with: error sending request",
              "path": [
                "me"
              ],
              "extensions": {
                "code": "SUBGRAPH_REQUEST_ERROR"
              }
            }
          ]
        }
        "###);
    })
}

#[test]
fn static_schema() {
    let schema = load_schema("big");

    let query = indoc! {r#"
        query Me {
          me {
            id
          }
        }
    "#};

    with_static_server("", &schema, None, None, |client| async move {
        let result: serde_json::Value = client.gql(query).send().await;
        let result = serde_json::to_string_pretty(&result).unwrap();

        insta::assert_snapshot!(&result, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Request to subgraph 'accounts' failed with: error sending request",
              "path": [
                "me"
              ],
              "extensions": {
                "code": "SUBGRAPH_REQUEST_ERROR"
              }
            }
          ]
        }
        "###);
    })
}

#[test]
fn introspect_enabled() {
    let config = indoc! {r#"
        [graph]
        introspection = true
    "#};

    let schema = load_schema("big");

    with_static_server(config, &schema, None, None, |client| async move {
        let result = introspect(client.endpoint()).await;

        insta::assert_snapshot!(&result, @r#"
        type Cart {
          products: [Product!]!
        }

        type Picture {
          url: String!
          width: Int!
          height: Int!
        }

        type Product {
          name: String!
          upc: String!
          price: Int!
          reviews: [Review!]!
        }

        type Query {
          me: User!
          topProducts: [Product!]!
        }

        type Review {
          id: ID!
          body: String!
          pictures: [Picture!]!
          product: Product!
          author: User
        }

        type Subscription {
          newProducts: Product!
        }

        enum Trustworthiness {
          REALLY_TRUSTED
          KINDA_TRUSTED
          NOT_TRUSTED
        }

        type User {
          id: ID!
          username: String!
          profilePicture: Picture

          """
          This used to be part of this subgraph, but is now being overridden from
          `reviews`
          """
          reviewCount: Int!
          joinedTimestamp: Int!
          cart: Cart!
          reviews: [Review!]!
          trustworthiness: Trustworthiness!
        }
        "#);
    })
}

#[test]
fn introspect_disabled() {
    let config = indoc! {r#"
        [graph]
        introspection = false
    "#};

    let schema = load_schema("big");

    with_static_server(config, &schema, None, None, |client| async move {
        let result = introspect(client.endpoint()).await;
        insta::assert_snapshot!(&result, @r###""###);
    })
}

#[test]
fn custom_path() {
    let config = indoc! {r#"
        [graph]
        path = "/custom"
    "#};

    let schema = load_schema("big");

    let query = indoc! {r#"
        query Me {
          me {
            id
          }
        }
    "#};

    with_static_server(config, &schema, Some("custom"), None, |client| async move {
        let result: serde_json::Value = client.gql(query).send().await;
        let result = serde_json::to_string_pretty(&result).unwrap();

        insta::assert_snapshot!(&result, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Request to subgraph 'accounts' failed with: error sending request",
              "path": [
                "me"
              ],
              "extensions": {
                "code": "SUBGRAPH_REQUEST_ERROR"
              }
            }
          ]
        }
        "###);
    })
}

#[test]
fn csrf_no_header() {
    let config = indoc! {r#"
        [csrf]
        enabled = true
    "#};

    let schema = load_schema("big");

    let query = indoc! {r#"
        query Me {
          me {
            id
          }
        }
    "#};

    with_static_server(config, &schema, Some("custom"), None, |client| async move {
        let response = client.gql::<serde_json::Value>(query).request().await;
        assert_eq!(http::StatusCode::FORBIDDEN, response.status());
    })
}

#[test]
fn csrf_with_header() {
    let config = indoc! {r#"
        [csrf]
        enabled = true
    "#};

    let schema = load_schema("big");

    let query = indoc! {r#"
        query Me {
          me {
            id
          }
        }
    "#};

    let headers = &[("x-grafbase-csrf-protection", "1")];

    with_static_server(config, &schema, None, Some(headers), |client| async move {
        let result: serde_json::Value = client.gql(query).send().await;
        let result = serde_json::to_string_pretty(&result).unwrap();

        insta::assert_snapshot!(&result, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Request to subgraph 'accounts' failed with: error sending request",
              "path": [
                "me"
              ],
              "extensions": {
                "code": "SUBGRAPH_REQUEST_ERROR"
              }
            }
          ]
        }
        "###);
    })
}

#[test]
fn hybrid_graph() {
    let schema = load_schema("big");

    let query = indoc! {r#"
        query Me {
          me {
            id
          }
        }
    "#};

    with_hybrid_server("", "test_graph", &schema, |client, _, _| async move {
        let result: serde_json::Value = client.gql(query).send().await;
        let result = serde_json::to_string_pretty(&result).unwrap();

        insta::assert_snapshot!(&result, @r###"
        {
          "data": null,
          "errors": [
            {
              "message": "Request to subgraph 'accounts' failed with: error sending request",
              "path": [
                "me"
              ],
              "extensions": {
                "code": "SUBGRAPH_REQUEST_ERROR"
              }
            }
          ]
        }
        "###);
    });
}

#[test]
fn health_default_config() {
    let config = "";
    let schema = load_schema("big");

    with_static_server(config, &schema, None, None, |client| async move {
        let mut url: reqwest::Url = client.endpoint().parse().unwrap();
        url.set_path("/health");

        let response = client.client().get(url).send().await.unwrap();

        assert_eq!(response.status(), 200);

        let body: serde_json::Value = response.json().await.unwrap();

        insta::assert_json_snapshot!(&body, @r###"
        {
          "status": "healthy"
        }
        "###);
    });
}

#[test]
fn health_custom_path() {
    let config = r#"
        [health]
        path = "/gezondheid"
    "#;

    let schema = load_schema("big");

    with_static_server(config, &schema, None, None, |client| async move {
        let mut url: reqwest::Url = client.endpoint().parse().unwrap();
        url.set_path("/health");

        let response = client.client().get(url.clone()).send().await.unwrap();

        assert_eq!(response.status(), 404);
        assert_eq!(response.text().await.unwrap(), "");

        // Now with the configured path
        url.set_path("/gezondheid");

        let response = client.client().get(url).send().await.unwrap();

        assert_eq!(response.status(), 200);

        let body: serde_json::Value = response.json().await.unwrap();

        insta::assert_json_snapshot!(&body, @r###"
        {
          "status": "healthy"
        }
        "###);
    });
}

#[test]
fn health_disabled() {
    let config = r#"
        [health]
        enabled = false
    "#;

    let schema = load_schema("big");

    with_static_server(config, &schema, None, None, |client| async move {
        let mut url: reqwest::Url = client.endpoint().parse().unwrap();
        url.set_path("/health");

        let response = client.client().get(url).send().await.unwrap();

        assert_eq!(response.status(), 404);
        assert_eq!(response.text().await.unwrap(), "");
    });
}

#[test]
fn health_custom_listener() {
    let config = r#"
        [health]
        path = "/gezondheid"
        listen = "0.0.0.0:9668"
    "#;

    let schema = load_schema("big");

    with_static_server(config, &schema, None, None, |client| async move {
        // First check that the health endpoint on the regular socket is not on.
        let mut url: reqwest::Url = client.endpoint().parse().unwrap();
        url.set_path("/health");

        let response = client.client().get(url).send().await.unwrap();

        assert_eq!(response.status(), 404);
        assert_eq!(response.text().await.unwrap(), "");

        // Then check at the configured port.

        let url: reqwest::Url = "http://127.0.0.1:9668/gezondheid".parse().unwrap();
        let response = client.client().get(url).send().await.unwrap();

        assert_eq!(response.status(), 200);

        let body: serde_json::Value = response.json().await.unwrap();

        insta::assert_json_snapshot!(&body, @r###"
        {
          "status": "healthy"
        }
        "###);
    });
}

#[test]
fn schema_file_hot_reload() {
    let config = indoc! {r#"
        [graph]
        introspection = true
    "#};

    let schema = load_schema("tiny");

    with_static_server(config, &schema, None, None, |client| async move {
        let result = introspect(client.endpoint()).await;

        insta::assert_snapshot!(&result, @r#"
        type Query {
          me: User!
        }

        type User {
          id: ID!
          username: String!
        }
        "#);

        let schema_path = client.schema_path().unwrap();

        std::fs::write(schema_path, load_schema("tiny_modified")).unwrap();
        std::thread::sleep(Duration::from_secs(6));

        let result = introspect(client.endpoint()).await;

        insta::assert_snapshot!(&result, @r#"
        type Query {
          me: User!
        }

        type User {
          id: ID!
        }
        "#);
    });
}

#[test]
fn global_rate_limiting() {
    let config = indoc! {r#"
        [gateway.rate_limit.global]
        limit = 1
        duration = "1s"
    "#};

    let schema = load_schema("big");

    let query = indoc! {r#"
        query Me {
          me {
            id
          }
        }
    "#};

    with_static_server(config, &schema, None, None, |client| async move {
        expect_rate_limiting(|| client.gql(query).send().boxed()).await;
    })
}

#[test]
fn subgraph_rate_limiting() {
    let config = indoc! {r#"
        [subgraphs.accounts.rate_limit]
        limit = 1
        duration = "1s"
    "#};

    let schema = load_schema("big");

    let query = indoc! {r#"
        query Me {
          me {
            id
          }
        }
    "#};

    with_static_server(config, &schema, None, None, |client| async move {
        expect_rate_limiting(|| client.gql(query).send().boxed()).await;
    })
}

#[test]
fn global_redis_rate_limiting() {
    let config = indoc! {r#"
        [gateway.rate_limit]
        storage = "redis"

        [gateway.rate_limit.global]
        limit = 1
        duration = "1s"
    "#};

    let schema = load_schema("big");

    let query = indoc! {r#"
        query Me {
          me {
            id
          }
        }
    "#};

    with_static_server(config, &schema, None, None, |client| async move {
        expect_rate_limiting(|| client.gql(query).send().boxed()).await;
    })
}

#[test]
fn subgraph_redis_rate_limiting() {
    let config = indoc! {r#"
        [gateway.rate_limit]
        storage = "redis"

        [subgraphs.accounts.rate_limit]
        limit = 1
        duration = "1s"
    "#};

    let schema = load_schema("big");

    let query = indoc! {r#"
        query Me {
          me {
            id
          }
        }
    "#};

    with_static_server(config, &schema, None, None, |client| async move {
        expect_rate_limiting(|| client.gql(query).send().boxed()).await;
    })
}

#[allow(clippy::panic)]
async fn expect_rate_limiting<'a, F>(f: F)
where
    F: Fn() -> BoxFuture<'a, serde_json::Value>,
{
    let destiny = Instant::now().checked_add(Duration::from_secs(60)).unwrap();

    loop {
        let response = Box::pin(f());
        let response = response.await;

        if response["errors"][0]["extensions"]["code"] == "RATE_LIMITED" {
            break;
        }

        if Instant::now().gt(&destiny) {
            panic!("Expected requests to get rate limited ...");
        }
    }
}
