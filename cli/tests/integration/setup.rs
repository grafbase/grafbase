mod test_extensions;

pub(crate) use self::test_extensions::*;

use std::{any::TypeId, borrow::Cow, fs, io::Write as _, net::SocketAddr, process, time::Duration};

use futures::{FutureExt as _, future::BoxFuture};
use graphql_mocks::MockGraphQlServer;
use rand::random;
use tokio::time::sleep;

#[derive(Default)]
pub(crate) struct GrafbaseDevConfig {
    mock_subgraphs: Vec<(TypeId, String, BoxFuture<'static, MockGraphQlServer>)>,
    sdl_only_subgraphs: Vec<(Cow<'static, str>, String)>,
    config: Option<Cow<'static, str>>,
}

impl GrafbaseDevConfig {
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub(crate) fn with_config(mut self, config: impl Into<Cow<'static, str>>) -> Self {
        self.config = Some(config.into());
        self
    }

    pub(crate) fn with_subgraph<S: graphql_mocks::Subgraph>(mut self, subgraph: S) -> Self {
        let name = subgraph.name();
        self.mock_subgraphs
            .push((std::any::TypeId::of::<S>(), name.to_string(), subgraph.start().boxed()));
        self
    }

    pub(crate) fn with_sdl_only_subgraph(mut self, subgraph_name: impl Into<Cow<'static, str>>, sdl: String) -> Self {
        self.sdl_only_subgraphs.push((subgraph_name.into(), sdl));
        self
    }

    pub(crate) async fn start(self) -> GrafbaseDev {
        let GrafbaseDevConfig {
            mock_subgraphs,
            sdl_only_subgraphs,
            config,
        } = self;

        let subgraphs =
            integration_tests::gateway::Subgraphs::load(mock_subgraphs, Default::default(), Default::default()).await;

        let working_directory = tempfile::tempdir().unwrap();

        let config_path = working_directory.path().join("grafbase.toml");
        let mut config_file = fs::File::create(&config_path).unwrap();
        if let Some(config) = config {
            writeln!(config_file, "{}", config.as_ref()).unwrap();
        };

        for subgraph in subgraphs.iter() {
            let sdl = subgraph.sdl();
            // Not necessary right now, but we may want to allow overriding.
            let schema_path = working_directory.path().join(format!("{}.graphql", subgraph.name()));
            let subgraph_url = subgraph.url().unwrap();
            fs::write(&schema_path, sdl.as_bytes()).unwrap();

            writeln!(
                config_file,
                r#"
                    [subgraphs.{name}]
                    url = "{subgraph_url}"
                    schema_path = '{schema_path}'
                "#,
                name = subgraph.name(),
                schema_path = schema_path.display(),
            )
            .unwrap();
        }

        for (subgraph_name, sdl) in sdl_only_subgraphs {
            let schema_path = working_directory.path().join(format!("{subgraph_name}.graphql"));
            fs::write(&schema_path, sdl.as_bytes()).unwrap();

            writeln!(
                config_file,
                r#"
                    [subgraphs.{name}]
                    schema_path = '{schema_path}'
                "#,
                name = subgraph_name,
                schema_path = schema_path.display(),
            )
            .unwrap();
        }

        // Pick a port number in the dynamic range.
        let port = random::<u16>() | 0xc000;

        let mut command = process::Command::new(crate::GRAFBASE_CLI_BIN_PATH);

        command
            .current_dir(working_directory.path())
            .arg("dev")
            .arg("--port")
            .arg(port.to_string())
            .arg("-c")
            .arg(config_path);

        let grafbase_process = command
            .stdout(process::Stdio::inherit())
            .stderr(process::Stdio::inherit())
            .spawn()
            .unwrap();

        let http_client = reqwest::Client::new();

        let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();

        let start = std::time::Instant::now();
        println!("Waiting for grafbase dev to be available at {addr}...");
        // On MacOS port mapping takes forever (with colima at least), but on Linux it's sub
        // millisecond. CI is however not fast enough for the whole gateway to be fully started
        // before the test starts. So ensuring we always give the gateway some time.
        sleep(Duration::from_millis(20)).await;

        while tokio::net::TcpStream::connect(addr).await.is_err() {
            sleep(Duration::from_millis(100)).await;
        }

        println!(
            "Could connect to socket after {} ms, waiting for HTTP server to be fully up and running.",
            start.elapsed().as_millis()
        );
        let url: url::Url = format!("http://{addr}/graphql").parse().unwrap();
        while !http_client
            .request(http::Method::OPTIONS, url.clone())
            .send()
            .await
            .is_ok_and(|resp| resp.status().is_success())
        {
            sleep(Duration::from_millis(100)).await;
        }

        println!("Waited for {} ms", start.elapsed().as_millis());

        GrafbaseDev {
            _subgraphs: subgraphs,
            port,
            _working_directory: working_directory,
            http_client,
            _grafbase_process: grafbase_process,
        }
    }
}

#[derive(serde::Serialize)]
pub(crate) struct GraphQlRequest<'a> {
    query: Cow<'a, str>,
    variables: Option<serde_json::Value>,
}

impl<'a> From<&'a str> for GraphQlRequest<'a> {
    fn from(value: &'a str) -> Self {
        GraphQlRequest {
            query: Cow::Borrowed(value),
            variables: None,
        }
    }
}

pub(crate) struct GrafbaseDev {
    _subgraphs: integration_tests::gateway::Subgraphs,
    _working_directory: tempfile::TempDir,
    _grafbase_process: process::Child,
    port: u16,
    http_client: reqwest::Client,
}

impl GrafbaseDev {
    pub(crate) async fn graphql_simple(&self, request: impl Into<GraphQlRequest<'_>>) -> serde_json::Value {
        let request: GraphQlRequest<'_> = request.into();
        let port = self.port;

        let url = format!("http://127.0.0.1:{port}/graphql");

        let response = self.http_client.post(url).json(&request).send().await.unwrap();

        assert!(response.status().is_success());

        response.json().await.unwrap()
    }
}

impl Drop for GrafbaseDev {
    fn drop(&mut self) {
        self._grafbase_process.kill().unwrap();
    }
}
