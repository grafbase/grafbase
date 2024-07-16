mod bench;
mod test_runtime;

use std::{collections::HashMap, sync::Arc};

use crate::{fetch::FetchRecorder, mock_trusted_documents::MockTrustedDocumentsClient, TestTrustedDocument};
use async_graphql_parser::types::ServiceDocument;
pub use bench::*;
use graphql_composition::FederatedGraph;
use graphql_federated_graph::FederatedGraphV3;
use graphql_mocks::MockGraphQlServer;
use parser_sdl::{connector_parsers::MockConnectorParsers, federation::header::SubgraphHeaderRule};
use runtime::{fetch::FetcherInner, hooks::DynamicHooks, trusted_documents_client};
pub use test_runtime::*;

use super::TestEngineV2;

#[must_use]
pub struct EngineV2Builder {
    federated_sdl: Option<String>,
    subgraphs: Vec<TestSubgraph>,
    config_sdl: Option<String>,
    runtime: TestRuntime,
    header_rules: Vec<SubgraphHeaderRule>,
}

struct TestSubgraph {
    name: String,
    url: String,
    schema: ServiceDocument,
}

pub trait EngineV2Ext {
    fn builder() -> EngineV2Builder {
        EngineV2Builder {
            federated_sdl: None,
            subgraphs: vec![],
            config_sdl: None,
            header_rules: Vec::new(),
            runtime: TestRuntime::default(),
        }
    }
}

impl EngineV2Ext for engine_v2::Engine<TestRuntime> {}

pub trait SchemaSource {
    fn sdl(&self) -> String;
    fn url(&self) -> String;
}

impl EngineV2Builder {
    pub fn with_supergraph_config(mut self, sdl: impl Into<String>) -> Self {
        self.config_sdl = Some(format!("{}\nextend schema @graph(type: federated)", sdl.into()));
        self
    }

    pub fn with_subgraph(mut self, name: &str, schema: &impl SchemaSource) -> Self {
        self.subgraphs.push(TestSubgraph {
            name: name.to_string(),
            url: schema.url(),
            schema: async_graphql_parser::parse_schema(schema.sdl()).expect("schema to be well formed"),
        });
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

    pub fn with_header_rule(mut self, rule: SubgraphHeaderRule) -> Self {
        self.header_rules.push(rule);
        self
    }

    pub fn with_fetcher(mut self, fetcher: impl FetcherInner + 'static) -> Self {
        self.runtime.fetcher = runtime::fetch::Fetcher::new(fetcher);
        self
    }

    pub async fn build(self) -> TestEngineV2 {
        let (config, runtime) = self.finalize().await;
        let fetcher = FetchRecorder::record(runtime.fetcher.clone()).with_url_to_subgraph_name(
            config
                .graph
                .subgraphs
                .iter()
                .map(|subgraph| {
                    let url = config.graph[subgraph.url].parse().unwrap();
                    let name = config.graph[subgraph.name].clone();
                    (url, name)
                })
                .collect(),
        );
        let recorded_subrequests = fetcher.recorded_requests();
        let engine = engine_v2::Engine::new(
            Arc::new(config.try_into().unwrap()),
            None,
            TestRuntime {
                fetcher: runtime::fetch::Fetcher::new(fetcher),
                ..runtime
            },
        )
        .await;

        TestEngineV2 {
            engine: Arc::new(engine),
            recorded_subrequests,
        }
    }

    async fn finalize(self) -> (engine_v2::config::Config, TestRuntime) {
        assert!(
            self.federated_sdl.is_none() || self.subgraphs.is_empty(),
            "Cannot use both subgraphs and federated SDL together"
        );

        let sdl = self.federated_sdl.unwrap_or_else(|| {
            let graph = if !self.subgraphs.is_empty() {
                let mut subgraphs = graphql_composition::Subgraphs::default();
                for TestSubgraph { name, url, schema } in &self.subgraphs {
                    subgraphs.ingest(schema, name, url);
                }
                graphql_composition::compose(&subgraphs)
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

        let mut federated_graph_config = match &self.config_sdl {
            Some(sdl) => {
                parser_sdl::parse(sdl, &HashMap::new(), &MockConnectorParsers::default())
                    .await
                    .expect("supergraph config SDL to be valid")
                    .federated_graph_config
            }
            None => None,
        }
        .unwrap_or_default();

        federated_graph_config.header_rules.extend(self.header_rules);
        (
            engine_config_builder::build_config(&federated_graph_config, graph).into_latest(),
            self.runtime,
        )
    }
}

#[async_trait::async_trait]
impl SchemaSource for String {
    fn sdl(&self) -> String {
        self.clone()
    }

    // Probably shouldn't really use this SchemaSource since this'll never work.
    fn url(&self) -> String {
        "http://example.com".to_string()
    }
}

#[async_trait::async_trait]
impl<T> SchemaSource for &T
where
    T: SchemaSource + Send + Sync,
{
    fn sdl(&self) -> String {
        T::sdl(self)
    }

    fn url(&self) -> String {
        T::url(self)
    }
}

#[async_trait::async_trait]
impl SchemaSource for MockGraphQlServer {
    fn sdl(&self) -> String {
        self.schema.sdl()
    }

    fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port())
    }
}
