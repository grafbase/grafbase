mod bench;
mod test_runtime;

use std::{collections::HashMap, sync::Arc, time::Duration};

use crate::{mock_trusted_documents::MockTrustedDocumentsClient, TestTrustedDocument};
pub use bench::*;
use futures::{future::BoxFuture, stream::FuturesUnordered, FutureExt, StreamExt};
use graphql_composition::FederatedGraph;
use graphql_federated_graph::FederatedGraphV3;
use graphql_mocks::MockGraphQlServer;
use parser_sdl::{connector_parsers::MockConnectorParsers, federation::header::SubgraphHeaderRule};
use runtime::{fetch::FetcherInner, hooks::DynamicHooks, trusted_documents_client};
use runtime_local::{ComponentLoader, HooksWasi, HooksWasiConfig};
pub use test_runtime::*;

use super::TestEngineV2;

enum Config {
    Sdl(String),
    SdlWebsocket,
}

#[must_use]
pub struct EngineV2Builder {
    federated_sdl: Option<String>,
    subgraphs: HashMap<std::any::TypeId, (String, BoxFuture<'static, MockGraphQlServer>)>,
    config: Option<Config>,
    runtime: TestRuntime,
    header_rules: Vec<SubgraphHeaderRule>,
    timeout: Option<Duration>,
    enable_entity_caching: bool,
}

pub trait EngineV2Ext {
    fn builder() -> EngineV2Builder {
        EngineV2Builder {
            federated_sdl: None,
            subgraphs: HashMap::new(),
            config: None,
            header_rules: Vec::new(),
            timeout: None,
            runtime: TestRuntime::default(),
            enable_entity_caching: false,
        }
    }
}

impl EngineV2Ext for engine_v2::Engine<TestRuntime> {}

impl EngineV2Builder {
    pub fn with_sdl_config(mut self, sdl: impl Into<String>) -> Self {
        self.config = Some(Config::Sdl(sdl.into()));
        self
    }

    pub fn with_sdl_websocket_config(mut self) -> Self {
        self.config = Some(Config::SdlWebsocket);
        self
    }

    pub fn with_subgraph<S: graphql_mocks::Subgraph>(mut self, subgraph: S) -> Self {
        let name = subgraph.name();
        self.subgraphs
            .insert(std::any::TypeId::of::<S>(), (name, subgraph.start().boxed()));
        self
    }

    pub fn with_federated_sdl(mut self, sdl: &str) -> Self {
        self.federated_sdl = Some(sdl.to_string());
        self
    }

    pub fn with_trusted_documents(mut self, branch_id: String, documents: Vec<TestTrustedDocument>) -> Self {
        self.runtime.trusted_documents = trusted_documents_client::Client::new(MockTrustedDocumentsClient {
            _branch_id: branch_id,
            documents,
        });
        self
    }

    pub fn with_hooks(mut self, hooks: impl Into<DynamicHooks>) -> Self {
        self.runtime.hooks = hooks.into();
        self
    }

    /// Wasm location will be assumed to be in our examples
    pub fn with_wasi_hooks(mut self, config: HooksWasiConfig) -> Self {
        let wasi_hooks = HooksWasi::new(Some(
            ComponentLoader::new(
                config.with_location_root_dir("../wasi-component-loader/examples/target/wasm32-wasip1/debug"),
            )
            .ok()
            .flatten()
            .expect("Wasm examples weren't built, please run:\ncd engine/crates/wasi-component-loader/examples && cargo component build"),
        ));
        self.runtime.hooks = DynamicHooks::wrap(wasi_hooks);
        self
    }

    pub fn with_header_rule(mut self, rule: SubgraphHeaderRule) -> Self {
        self.header_rules.push(rule);
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn with_entity_caching(mut self) -> Self {
        self.enable_entity_caching = true;
        self
    }

    pub fn with_fetcher(mut self, fetcher: impl FetcherInner + 'static) -> Self {
        self.runtime.fetcher = runtime::fetch::Fetcher::new(fetcher);
        self
    }

    pub async fn build(self) -> TestEngineV2 {
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

        let sdl = self.federated_sdl.unwrap_or_else(|| {
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
            graph.into_sdl().unwrap()
        });

        // Ensure SDL/JSON serialization work as a expected
        let graph = FederatedGraph::from_sdl(&sdl).unwrap();
        let graph = serde_json::from_value(serde_json::to_value(&graph).unwrap()).unwrap();

        let mut federated_graph_config = match self.config {
            Some(Config::Sdl(mut sdl)) => {
                sdl.push_str("\nextend schema @graph(type: federated)");
                parser_sdl::parse(&sdl, &HashMap::new(), &MockConnectorParsers::default())
                    .await
                    .expect("supergraph config SDL to be valid")
                    .federated_graph_config
            }
            Some(Config::SdlWebsocket) => {
                let mut sdl = String::new();
                sdl.push_str("\nextend schema @graph(type: federated)");
                for (_, subgraph) in &subgraphs {
                    sdl.push_str(&format!(
                        "\nextend schema @subgraph(name: \"{}\", websocketUrl: \"{}\")",
                        subgraph.name,
                        subgraph.websocket_url()
                    ));
                }

                parser_sdl::parse(&sdl, &HashMap::new(), &MockConnectorParsers::default())
                    .await
                    .expect("supergraph config SDL to be valid")
                    .federated_graph_config
            }
            None => None,
        }
        .unwrap_or_default();

        federated_graph_config.timeout = self.timeout;
        federated_graph_config.header_rules.extend(self.header_rules);
        federated_graph_config.enable_entity_caching = self.enable_entity_caching;

        let config = engine_config_builder::build_config(&federated_graph_config, graph).into_latest();
        let engine = engine_v2::Engine::new(Arc::new(config.try_into().unwrap()), None, self.runtime).await;

        TestEngineV2 {
            engine: Arc::new(engine),
            subgraphs: subgraphs.into_iter().collect(),
        }
    }
}
