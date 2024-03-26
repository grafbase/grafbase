#![allow(unused_crate_dependencies)]

use std::{
    env, fs,
    marker::PhantomData,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    panic::{catch_unwind, AssertUnwindSafe},
    path::{self, Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex, OnceLock},
    time::{Duration, SystemTime},
};

use duct::{cmd, Handle};
use futures_util::{Future, FutureExt};
use http::{HeaderMap, StatusCode};
use licensing::ES256KeyPair;
use tempfile::{tempdir, TempDir};
use tokio::runtime::Runtime;
use wiremock::{
    matchers::{header, method, path},
    Mock, ResponseTemplate,
};

pub fn private_key() -> &'static ES256KeyPair {
    static PRIVATE_KEY: OnceLock<ES256KeyPair> = OnceLock::new();

    PRIVATE_KEY.get_or_init(|| {
        let output = Command::new(env!("CARGO"))
            .arg("locate-project")
            .arg("--workspace")
            .arg("--message-format=plain")
            .output()
            .unwrap()
            .stdout;

        let cargo_path = Path::new(std::str::from_utf8(&output).unwrap().trim());
        let workspace_dir = cargo_path.parent().unwrap().to_path_buf();
        let private_key_path = workspace_dir.join("engine/crates/licensing/test/private-test-key.pem");

        let pem = fs::read_to_string(private_key_path).unwrap();
        ES256KeyPair::from_pem(&pem).unwrap()
    })
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

pub const ACCESS_TOKEN: &str = "test";

pub struct Client {
    endpoint: String,
    client: reqwest::Client,
    headers: HeaderMap,
    commands: CommandHandles,
}

impl Client {
    pub fn new(endpoint: String, commands: CommandHandles) -> Self {
        Self {
            endpoint,
            headers: HeaderMap::new(),
            client: reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(1))
                .build()
                .unwrap(),
            commands,
        }
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
            reqwest_builder,
            bearer: None,
        }
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
    const INITIAL_PORT: u16 = 4000;

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

pub fn listen_address() -> SocketAddr {
    let port = get_free_port();
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port))
}

#[derive(Debug)]
pub struct Project {
    _temp_dir: TempDir,
    pub config_path: PathBuf,
    pub schema_path: Option<PathBuf>,
    pub license_path: Option<PathBuf>,
}

pub fn with_config_files(config: &str, schema: Option<&str>, license: Option<&str>) -> Project {
    let temp_dir = tempdir().unwrap();

    let config_path = temp_dir.path().join("grafbase.toml");
    fs::write(&config_path, config).unwrap();

    let schema_path = match schema {
        Some(schema) => {
            let schema_path = temp_dir.path().join("schema.graphql");
            fs::write(&schema_path, schema).unwrap();

            Some(schema_path)
        }
        None => None,
    };

    let license_path = match license {
        Some(license) => {
            let license_path = temp_dir.path().join("license");
            fs::write(&license_path, license).unwrap();

            Some(license_path)
        }
        None => None,
    };

    Project {
        config_path,
        schema_path,
        license_path,
        _temp_dir: temp_dir,
    }
}

pub fn with_static_server<F, T>(
    config: &str,
    schema: &str,
    path: Option<&str>,
    headers: Option<&'static [(&'static str, &'static str)]>,
    license: Option<&str>,
    test: T,
) where
    T: FnOnce(Arc<Client>) -> F,
    F: Future<Output = ()>,
{
    let project = with_config_files(config, Some(schema), license);
    let addr = listen_address();

    let command = match project.license_path {
        Some(license_path) => cmd!(
            cargo_bin("grafbase-gateway"),
            "--listen-address",
            &addr.to_string(),
            "--config",
            &project.config_path.to_str().unwrap(),
            "--schema",
            &project.schema_path.unwrap().to_str().unwrap(),
            "--license",
            &license_path.to_str().unwrap(),
        ),
        None => cmd!(
            cargo_bin("grafbase-gateway"),
            "--listen-address",
            &addr.to_string(),
            "--config",
            &project.config_path.to_str().unwrap(),
            "--schema",
            &project.schema_path.unwrap().to_str().unwrap(),
        ),
    };

    let endpoint = match path {
        Some(path) => format!("http://{addr}/{path}"),
        None => format!("http://{addr}/graphql"),
    };

    let mut commands = CommandHandles::new();
    commands.push(command.start().unwrap());

    let mut client = Client::new(endpoint, commands);

    if let Some(headers) = headers {
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

    res.unwrap();
}

pub fn with_hybrid_server<F, T>(config: &str, graph_ref: &str, sdl: &str, test: T)
where
    T: FnOnce(Arc<Client>) -> F,
    F: Future<Output = ()>,
{
    let project = with_config_files(config, None, None);
    let addr = listen_address();

    let uplink_response = serde_json::json!({
        "account_id": "01HR7NP3A4NDVWC10PZW6ZMC5P",
        "graph_id": "01HR7NPB8E3YW29S5PPSY1AQKR",
        "branch": "main",
        "branch_id": "01HR7NPB8E3YW29S5PPSY1AQKA",
        "sdl": sdl,
        "version_id": "01HR7NPYWWM6DEKACKKN3EPFP2",
    });

    let res = runtime().block_on(async {
        let response = ResponseTemplate::new(200).set_body_string(serde_json::to_string(&uplink_response).unwrap());
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
            &project.config_path.to_str().unwrap(),
            "--graph-ref",
            graph_ref,
        )
        .env("GRAFBASE_GDN_URL", format!("http://{}", server.address()))
        .env("GRAFBASE_ACCESS_TOKEN", ACCESS_TOKEN);

        let mut commands = CommandHandles::new();
        commands.push(command.start().unwrap());

        let client = Arc::new(Client::new(format!("http://{addr}/graphql"), commands));

        client.poll_endpoint(30, 300).await;

        let res = AssertUnwindSafe(test(client.clone())).catch_unwind().await;

        client.kill_handles();

        res
    });

    res.unwrap();
}

pub fn load_schema(name: &str) -> String {
    let path = format!("./tests/schemas/{name}.graphql");
    fs::read_to_string(path).unwrap()
}

pub async fn introspect(url: &str) -> String {
    grafbase_graphql_introspection::introspect(url, &[("x-api-key", "")])
        .await
        .unwrap_or_default()
}
