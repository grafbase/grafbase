use std::{any::TypeId, borrow::Cow, fs, io::Write as _, process, time::Duration};

use futures::{future::BoxFuture, FutureExt as _};
use graphql_mocks::MockGraphQlServer;
use rand::random;

#[derive(Default)]
pub(crate) struct GrafbaseDevConfig {
    mock_subgraphs: Vec<(TypeId, String, BoxFuture<'static, MockGraphQlServer>)>,
}

impl GrafbaseDevConfig {
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub(crate) fn with_subgraph<S: graphql_mocks::Subgraph>(mut self, subgraph: S) -> Self {
        let name = subgraph.name();
        self.mock_subgraphs
            .push((std::any::TypeId::of::<S>(), name.to_string(), subgraph.start().boxed()));
        self
    }

    pub(crate) async fn start(self) -> GrafbaseDev {
        let GrafbaseDevConfig { mock_subgraphs } = self;

        let subgraphs = integration_tests::federation::Subgraphs::load(mock_subgraphs, Default::default()).await;

        let working_directory = tempfile::tempdir().unwrap();

        let graph_overrides = {
            let path = working_directory.path().join("graph-overrides.toml");
            let mut out = fs::File::create(&path).unwrap();

            for subgraph in subgraphs.iter() {
                let sdl = subgraph.sdl();
                // Not necessary right now, but we may want to allow overriding.
                let schema_path = working_directory.path().join(format!("{}.graphql", subgraph.name()));
                let subgraph_url = subgraph.url();
                fs::write(&schema_path, sdl.as_bytes()).unwrap();

                writeln!(
                    out,
                    r#"
                    [subgraphs.{name}]
                    url = "{subgraph_url}"
                    schema_path = "{schema_path}"
                "#,
                    name = subgraph.name(),
                    schema_path = schema_path.display(),
                )
                .unwrap();
            }

            path
        };

        // Pick a port number in the dynamic range.
        let port = random::<u16>() | 0xc000;

        assert!(working_directory.path().exists());
        let canonicalized = std::fs::canonicalize(working_directory.path()).unwrap();

        assert!(graph_overrides.exists());

        let grafbase_process = process::Command::new(crate::GRAFBASE_CLI_BIN_PATH)
            .current_dir(canonicalized)
            .arg("dev")
            .arg("--port")
            .arg(port.to_string())
            .arg("--graph-overrides")
            .arg("graph-overrides.toml")
            .stdout(process::Stdio::inherit())
            .stderr(process::Stdio::inherit())
            .spawn()
            .unwrap();

        let http_client = reqwest::Client::new();

        // We have to sleep to allow the process to start up and pick up the configuration.
        tokio::time::sleep(Duration::from_millis(200)).await;

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
    _subgraphs: integration_tests::federation::Subgraphs,
    _working_directory: tempfile::TempDir,
    _grafbase_process: process::Child,
    port: u16,
    http_client: reqwest::Client,
}

impl GrafbaseDev {
    pub(crate) async fn graphql_simple(&self, request: impl Into<GraphQlRequest<'_>>) -> serde_json::Value {
        let request: GraphQlRequest<'_> = request.into();
        let port = self.port;

        let url = format!("http://localhost:{port}/graphql");

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
