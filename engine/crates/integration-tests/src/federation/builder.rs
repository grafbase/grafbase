mod bench;
mod test_runtime;

use std::{collections::HashMap, sync::Arc};

use crate::{mock_trusted_documents::MockTrustedDocumentsClient, TestTrustedDocument};
pub use bench::*;
use engine_config_builder::{build_with_sdl_config, build_with_toml_config};
use federated_graph::FederatedGraphV3;
use futures::{future::BoxFuture, stream::FuturesUnordered, FutureExt, StreamExt};
use graphql_composition::FederatedGraph;
use graphql_mocks::MockGraphQlServer;
use parser_sdl::{connector_parsers::MockConnectorParsers, federation::FederatedGraphConfig};
use runtime::{fetch::FetcherInner, hooks::DynamicHooks, trusted_documents_client};
use runtime_local::{ComponentLoader, HooksWasi};
pub use test_runtime::*;

use super::TestEngineV2;

enum ConfigSource {
    Sdl(String),
    Toml(String),
    SdlWebsocket,
}

#[must_use]
pub struct EngineV2Builder {
    federated_sdl: Option<String>,
    subgraphs: HashMap<std::any::TypeId, (String, BoxFuture<'static, MockGraphQlServer>)>,
    config_source: Option<ConfigSource>,
    runtime: TestRuntime,
}

pub trait EngineV2Ext {
    fn builder() -> EngineV2Builder {
        EngineV2Builder {
            federated_sdl: None,
            subgraphs: HashMap::new(),
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
        self.subgraphs
            .insert(std::any::TypeId::of::<S>(), (name, subgraph.start().boxed()));
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

    pub fn with_mock_fetcher(mut self, fetcher: impl FetcherInner + 'static) -> Self {
        self.runtime.fetcher = runtime::fetch::Fetcher::new(fetcher);
        self
    }
    //-- Runtime customization --

    pub async fn build(mut self) -> TestEngineV2 {
        let mut subgraphs = self
            .subgraphs
            .into_iter()
            .map(|(type_id, (name, server))| async move {
                (
                    type_id,
                    super::Subgraph {
                        name,
                        server: server.await,
                    },
                )
            })
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>()
            .await;
        // Ensures consistency of composition and thus introspection tests.
        subgraphs.sort_unstable_by(|a, b| a.1.name.cmp(&b.1.name));

        let mut graph = self
            .federated_sdl
            .map(|sdl| FederatedGraph::from_sdl(&sdl).unwrap())
            .unwrap_or_else(|| {
                let graph = if !subgraphs.is_empty() {
                    graphql_composition::compose(&subgraphs.iter().fold(
                        graphql_composition::Subgraphs::default(),
                        |mut subgraphs, (_, subgraph)| {
                            let schema =
                                async_graphql_parser::parse_schema(subgraph.sdl()).expect("schema to be well formed");
                            subgraphs.ingest(&schema, &subgraph.name, &subgraph.url());
                            subgraphs
                        },
                    ))
                    .into_result()
                    .expect("schemas to compose succesfully")
                } else {
                    FederatedGraph::V3(FederatedGraphV3::default())
                };
                graph
            });

        // Ensure SDL/JSON serialization work as a expected
        graph = FederatedGraph::from_sdl(&graph.into_sdl().unwrap()).unwrap();
        graph = serde_json::from_value(serde_json::to_value(&graph).unwrap()).unwrap();

        let config = match self.config_source {
            Some(ConfigSource::Toml(toml)) => {
                let config: gateway_config::Config = toml::from_str(&toml).unwrap();
                update_runtime_with_toml_config(&mut self.runtime, &config);
                build_with_toml_config(&config, graph)
            }
            Some(ConfigSource::Sdl(mut sdl)) => {
                sdl.push_str("\nextend schema @graph(type: federated)");
                let config = parse_sdl_config(&sdl).await;
                build_with_sdl_config(&config, graph)
            }
            Some(ConfigSource::SdlWebsocket) => {
                let mut sdl = String::new();
                sdl.push_str("\nextend schema @graph(type: federated)");
                for (_, subgraph) in &subgraphs {
                    sdl.push_str(&format!(
                        "\nextend schema @subgraph(name: \"{}\", websocketUrl: \"{}\")",
                        subgraph.name,
                        subgraph.websocket_url()
                    ));
                }

                let config = parse_sdl_config(&sdl).await;
                build_with_sdl_config(&config, graph)
            }
            None => build_with_sdl_config(&Default::default(), graph),
        }
        .into_latest();

        let engine = engine_v2::Engine::new(Arc::new(config.try_into().unwrap()), None, self.runtime).await;

        TestEngineV2 {
            engine: Arc::new(engine),
            subgraphs: subgraphs.into_iter().collect(),
        }
    }
}

fn update_runtime_with_toml_config(runtime: &mut TestRuntime, config: &gateway_config::Config) {
    if let Some(hooks_config) = config.hooks.clone() {
        let wasi_hooks = HooksWasi::new(Some(
                        ComponentLoader::new(
                            hooks_config
                        )
                        .ok()
                        .flatten()
                        .expect("Wasm examples weren't built, please run:\ncd engine/crates/wasi-component-loader/examples && cargo component build"),
                    ));
        runtime.hooks = DynamicHooks::wrap(wasi_hooks);
    }
}

async fn parse_sdl_config(sdl: &str) -> FederatedGraphConfig {
    parser_sdl::parse(sdl, &HashMap::new(), &MockConnectorParsers::default())
        .await
        .expect("supergraph config SDL to be valid")
        .federated_graph_config
        .unwrap_or_default()
}
