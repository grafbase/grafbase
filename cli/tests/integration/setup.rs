mod test_extensions;

pub(crate) use self::test_extensions::*;

use std::{any::TypeId, borrow::Cow, fs, io::Write as _, process, time::Duration};

use futures::{FutureExt as _, future::BoxFuture};
use graphql_mocks::MockGraphQlServer;
use rand::random;

#[derive(Default)]
pub(crate) struct GrafbaseDevConfig {
    mock_subgraphs: Vec<(TypeId, String, BoxFuture<'static, MockGraphQlServer>)>,
    sdl_only_subgraphs: Vec<(Cow<'static, str>, String)>,
    gateway_config: Option<Cow<'static, str>>,
}

impl GrafbaseDevConfig {
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub(crate) fn with_gateway_config(mut self, gateway_config: impl Into<Cow<'static, str>>) -> Self {
        self.gateway_config = Some(gateway_config.into());
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
            gateway_config,
        } = self;

        let subgraphs =
            integration_tests::federation::Subgraphs::load(mock_subgraphs, Default::default(), Default::default())
                .await;

        let working_directory = tempfile::tempdir().unwrap();

        let gateway_config_path = if let Some(gateway_config) = gateway_config {
            let path = working_directory.path().join("grafbase.toml");
            fs::write(&path, gateway_config.as_bytes()).unwrap();
            Some(path)
        } else {
            None
        };

        let _graph_overrides = {
            let path = working_directory.path().join("graph-overrides.toml");
            let mut out = fs::File::create(&path).unwrap();

            for subgraph in subgraphs.iter() {
                let sdl = subgraph.sdl();
                // Not necessary right now, but we may want to allow overriding.
                let schema_path = working_directory.path().join(format!("{}.graphql", subgraph.name()));
                let subgraph_url = subgraph.url().unwrap();
                fs::write(&schema_path, sdl.as_bytes()).unwrap();

                writeln!(
                    out,
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
                let schema_path = working_directory.path().join(format!("{}.graphql", subgraph_name));
                fs::write(&schema_path, sdl.as_bytes()).unwrap();

                writeln!(
                    out,
                    r#"
                    [subgraphs.{name}]
                    schema_path = '{schema_path}'
                "#,
                    name = subgraph_name,
                    schema_path = schema_path.display(),
                )
                .unwrap();
            }

            path
        };

        // Pick a port number in the dynamic range.
        let port = random::<u16>() | 0xc000;

        let mut command = process::Command::new(crate::GRAFBASE_CLI_BIN_PATH);

        command
            .current_dir(working_directory.path())
            .arg("dev")
            .arg("--port")
            .arg(port.to_string())
            .arg("--graph-overrides")
            .arg("graph-overrides.toml");

        if let Some(gateway_config_path) = gateway_config_path {
            command.arg("--gateway-config").arg(gateway_config_path);
        }

        let grafbase_process = command
            .stdout(process::Stdio::inherit())
            .stderr(process::Stdio::inherit())
            .spawn()
            .unwrap();

        let http_client = reqwest::Client::new();

        // We have to sleep to allow the process to start up and pick up the configuration.
        tokio::time::sleep(Duration::from_millis(2800)).await;

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
