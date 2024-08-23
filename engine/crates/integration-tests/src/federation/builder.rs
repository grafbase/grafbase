mod bench;
mod test_runtime;

use std::{
    any::TypeId,
    borrow::Cow,
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::{mock_trusted_documents::MockTrustedDocumentsClient, TestTrustedDocument};
pub use bench::*;
use cynic_introspection::{IntrospectionQuery, SpecificationVersion};
use engine_config_builder::{build_with_sdl_config, build_with_toml_config};
use federated_graph::FederatedGraphV3;
use futures::{future::BoxFuture, stream::FuturesUnordered, FutureExt, StreamExt};
use grafbase_telemetry::metrics::meter_from_global_provider;
use graphql_composition::FederatedGraph;
use graphql_mocks::MockGraphQlServer;
use itertools::Itertools;
use parser_sdl::{connector_parsers::MockConnectorParsers, federation::FederatedGraphConfig};
use runtime::{fetch::dynamic::DynamicFetcher, hooks::DynamicHooks, trusted_documents_client};
use runtime_local::{ComponentLoader, HooksWasi};
pub use test_runtime::*;
use url::Url;

use super::{DockerSubgraph, MockSubgraph, TestEngineV2};

enum ConfigSource {
    Sdl(String),
    Toml(String),
    SdlWebsocket,
}

#[must_use]
pub struct EngineV2Builder {
    federated_sdl: Option<String>,
    mock_subgraphs: Vec<(TypeId, String, BoxFuture<'static, MockGraphQlServer>)>,
    docker_subgraphs: HashSet<DockerSubgraph>,
    config_source: Option<ConfigSource>,
    runtime: TestRuntime,
}

pub trait EngineV2Ext {
    fn builder() -> EngineV2Builder {
        EngineV2Builder {
            federated_sdl: None,
            mock_subgraphs: Vec::new(),
            docker_subgraphs: HashSet::new(),
            config_source: None,
            runtime: TestRuntime::default(),
        }
    }
}

impl EngineV2Ext for engine_v2::Engine<TestRuntime> {}

impl EngineV2Builder {
    pub fn with_sdl_config(mut self, sdl: impl Into<String>) -> Self {
        assert!(self.config_source.is_none(), "overwriting config!");
        self.config_source = Some(ConfigSource::Sdl(sdl.into()));
        self
    }

    pub fn with_toml_config(mut self, toml: impl Into<String>) -> Self {
        assert!(self.config_source.is_none(), "overwriting config!");
        self.config_source = Some(ConfigSource::Toml(toml.into()));
        self
    }

    pub fn with_sdl_websocket_config(mut self) -> Self {
        assert!(self.config_source.is_none(), "overwriting config!");
        self.config_source = Some(ConfigSource::SdlWebsocket);
        self
    }

    pub fn with_subgraph<S: graphql_mocks::Subgraph>(mut self, subgraph: S) -> Self {
        let name = subgraph.name();
        self.mock_subgraphs
            .push((std::any::TypeId::of::<S>(), name.to_string(), subgraph.start().boxed()));
        self
    }

    pub fn with_docker_subgraph(mut self, subgraph: DockerSubgraph) -> Self {
        self.docker_subgraphs.insert(subgraph);
        self
    }

    /// Will bypass the composition of subgraphs and be used at its stead.
    pub fn with_federated_sdl(mut self, sdl: &str) -> Self {
        self.federated_sdl = Some(sdl.to_string());
        self
    }

    //-- Runtime customization --
    // Prefer passing through either the TOML / SDL config when relevant, see update_runtime_with_toml_config
    //--

    pub fn with_mock_trusted_documents(mut self, branch_id: String, documents: Vec<TestTrustedDocument>) -> Self {
        self.runtime.trusted_documents = trusted_documents_client::Client::new(MockTrustedDocumentsClient {
            _branch_id: branch_id,
            documents,
        });
        self
    }

    pub fn with_mock_hooks(mut self, hooks: impl Into<DynamicHooks>) -> Self {
        self.runtime.hooks = hooks.into();
        self
    }

    pub fn with_mock_fetcher(mut self, fetcher: impl Into<DynamicFetcher>) -> Self {
        self.runtime.fetcher = fetcher.into();
        self
    }
    //-- Runtime customization --

    pub async fn build(mut self) -> TestEngineV2 {
        let subgraphs = load_subgraphs(self.mock_subgraphs, self.docker_subgraphs).await;

        let graph = self
            .federated_sdl
            .map(|sdl| FederatedGraph::from_sdl(&sdl).unwrap())
            .unwrap_or_else(|| {
                if !subgraphs.is_empty() {
                    graphql_composition::compose(&subgraphs.iter().fold(
                        graphql_composition::Subgraphs::default(),
                        |mut subgraphs, subgraph| {
                            let schema =
                                async_graphql_parser::parse_schema(subgraph.sdl()).expect("schema to be well formed");
                            subgraphs.ingest(&schema, subgraph.name(), subgraph.url().as_ref());
                            subgraphs
                        },
                    ))
                    .into_result()
                    .expect("schemas to compose succesfully")
                } else {
                    FederatedGraph::V3(FederatedGraphV3::default())
                }
            });

        // Ensure SDL/JSON serialization work as a expected
        let graph = {
            let sdl = graph.into_federated_sdl();
            println!("{sdl}");
            let mut graph = FederatedGraph::from_sdl(&sdl).unwrap();
            let json = serde_json::to_value(&graph).unwrap();
            graph = serde_json::from_value(json).unwrap();
            graph
        };

        let config = match self.config_source {
            Some(ConfigSource::Toml(toml)) => {
                let config: gateway_config::Config = toml::from_str(&toml).unwrap();
                update_runtime_with_toml_config(&mut self.runtime, &config);
                build_with_toml_config(&config, graph.into_latest())
            }
            Some(ConfigSource::Sdl(mut sdl)) => {
                sdl.push_str("\nextend schema @graph(type: federated)");
                let config = parse_sdl_config(&sdl).await;
                build_with_sdl_config(&config, graph.into_latest())
            }
            Some(ConfigSource::SdlWebsocket) => {
                let mut sdl = String::new();
                sdl.push_str("\nextend schema @graph(type: federated)");
                for subgraph in &subgraphs {
                    sdl.push_str(&format!(
                        "\nextend schema @subgraph(name: \"{}\", websocketUrl: \"{}\")",
                        subgraph.name(),
                        subgraph.websocket_url()
                    ));
                }

                let config = parse_sdl_config(&sdl).await;
                build_with_sdl_config(&config, graph.into_latest())
            }
            None => build_with_sdl_config(&Default::default(), graph.into_latest()),
        }
        .into_latest();

        let engine = engine_v2::Engine::new(Arc::new(config.try_into().unwrap()), None, self.runtime).await;

        let (mock_subgraphs, docker_subgraphs) = subgraphs
            .into_iter()
            .map(|subgraph| match subgraph {
                Subgraph::Mock { type_id, server } => Ok((type_id, server)),
                Subgraph::Docker { subgraph, .. } => Err(subgraph),
            })
            .partition_result();
        TestEngineV2 {
            engine: Arc::new(engine),
            mock_subgraphs,
            docker_subgraphs,
        }
    }
}

fn update_runtime_with_toml_config(runtime: &mut TestRuntime, config: &gateway_config::Config) {
    if let Some(hooks_config) = config.hooks.clone() {
        let loader = ComponentLoader::new(hooks_config)
            .ok()
            .flatten()
            .expect("Wasm examples weren't built, please run:\ncd engine/crates/wasi-component-loader/examples && cargo component build");

        let meter = meter_from_global_provider();
        runtime.hooks = DynamicHooks::wrap(HooksWasi::new(Some(loader), &meter));
    }
}

async fn parse_sdl_config(sdl: &str) -> FederatedGraphConfig {
    parser_sdl::parse(sdl, &HashMap::new(), &MockConnectorParsers::default())
        .await
        .expect("supergraph config SDL to be valid")
        .federated_graph_config
        .unwrap_or_default()
}

async fn load_subgraphs(
    mock_subgraphs: Vec<(TypeId, String, BoxFuture<'static, MockGraphQlServer>)>,
    docker_subgraphs: HashSet<DockerSubgraph>,
) -> Vec<Subgraph> {
    let mock_subgraphs_fut = mock_subgraphs
        .into_iter()
        .map(|(type_id, name, server)| async move {
            Subgraph::Mock {
                type_id,
                server: MockSubgraph {
                    name,
                    server: server.await,
                },
            }
        })
        .collect::<FuturesUnordered<_>>()
        .collect::<Vec<_>>();

    let docker_subgraphs_fut = docker_subgraphs
        .into_iter()
        .map(|subgraph| async move {
            let request = IntrospectionQuery::with_capabilities(SpecificationVersion::October2021.capabilities());
            #[derive(serde::Deserialize)]
            struct Response {
                data: IntrospectionQuery,
            }
            let sdl = reqwest::Client::new()
                .post(subgraph.url())
                .json(&request)
                .send()
                .await
                .unwrap()
                .error_for_status()
                .unwrap()
                .json::<Response>()
                .await
                .unwrap()
                .data
                .into_schema()
                .unwrap()
                .to_sdl();
            Subgraph::Docker { sdl, subgraph }
        })
        .collect::<FuturesUnordered<_>>()
        .collect::<Vec<_>>();

    let (mut subgraphs, docker_subgraphs) = futures::join!(mock_subgraphs_fut, docker_subgraphs_fut);
    subgraphs.extend(docker_subgraphs);

    // Ensures consistency of composition and thus introspection tests.
    subgraphs.sort_unstable_by(|a, b| a.name().cmp(b.name()));

    subgraphs
}

enum Subgraph {
    Mock { type_id: TypeId, server: MockSubgraph },
    Docker { subgraph: DockerSubgraph, sdl: String },
}

impl Subgraph {
    fn name(&self) -> &str {
        match self {
            Subgraph::Mock { server, .. } => &server.name,
            Subgraph::Docker { subgraph, .. } => subgraph.name(),
        }
    }

    pub fn sdl(&self) -> Cow<'_, str> {
        match self {
            Subgraph::Mock { server, .. } => server.sdl().into(),
            Subgraph::Docker { sdl, .. } => sdl.into(),
        }
    }

    pub fn url(&self) -> Url {
        match self {
            Subgraph::Mock { server, .. } => server.url(),
            Subgraph::Docker { subgraph, .. } => subgraph.url(),
        }
    }

    pub fn websocket_url(&self) -> Url {
        match self {
            Subgraph::Mock { server, .. } => server.websocket_url(),
            Subgraph::Docker { subgraph, .. } => subgraph.url(),
        }
    }
}
