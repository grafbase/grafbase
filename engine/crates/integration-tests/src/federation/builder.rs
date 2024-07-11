mod bench;
mod mock;
mod test_runtime;

use std::{collections::HashMap, sync::Arc};

use crate::{mock_trusted_documents::MockTrustedDocumentsClient, TestTrustedDocument};
use async_graphql_parser::types::ServiceDocument;
pub use bench::*;
use graphql_composition::FederatedGraph;
use graphql_mocks::MockGraphQlServer;
pub use mock::*;
use parser_sdl::{connector_parsers::MockConnectorParsers, federation::header::SubgraphHeaderRule};
use runtime::{hooks::DynamicHooks, trusted_documents_client};
pub use test_runtime::*;

use super::TestFederationEngine;

#[must_use]
pub struct FederationGatewayBuilder {
    schemas: Vec<(String, String, ServiceDocument)>,
    trusted_documents: Option<MockTrustedDocumentsClient>,
    config_sdl: Option<String>,
    hooks: DynamicHooks,
    header_rules: Vec<SubgraphHeaderRule>,
}

pub trait GatewayV2Ext {
    fn builder() -> FederationGatewayBuilder {
        FederationGatewayBuilder {
            trusted_documents: None,
            schemas: vec![],
            config_sdl: None,
            hooks: DynamicHooks::default(),
            header_rules: Vec::new(),
        }
    }
}

impl GatewayV2Ext for engine_v2::Engine<TestRuntime> {}

#[async_trait::async_trait]
pub trait SchemaSource {
    async fn sdl(&self) -> String;
    fn url(&self) -> String;
}

impl FederationGatewayBuilder {
    pub fn with_supergraph_config(mut self, sdl: impl Into<String>) -> Self {
        self.config_sdl = Some(format!("{}\nextend schema @graph(type: federated)", sdl.into()));
        self
    }

    pub async fn with_schema(mut self, name: &str, schema: &impl SchemaSource) -> Self {
        self.schemas.push((
            name.to_string(),
            schema.url(),
            async_graphql_parser::parse_schema(schema.sdl().await).expect("schema to be well formed"),
        ));
        self
    }

    pub fn with_trusted_documents(mut self, branch_id: String, documents: Vec<TestTrustedDocument>) -> Self {
        self.trusted_documents = Some(MockTrustedDocumentsClient {
            _branch_id: branch_id,
            documents,
        });
        self
    }

    pub fn with_hooks(mut self, hooks: impl Into<DynamicHooks>) -> Self {
        self.hooks = hooks.into();
        self
    }

    pub fn with_header_rule(mut self, rule: SubgraphHeaderRule) -> Self {
        self.header_rules.push(rule);
        self
    }

    pub async fn finish(self) -> TestFederationEngine {
        let mut subgraphs = graphql_composition::Subgraphs::default();
        for (name, url, schema) in self.schemas {
            subgraphs.ingest(&schema, &name, &url);
        }
        let graph = graphql_composition::compose(&subgraphs)
            .into_result()
            .expect("schemas to compose succesfully");

        let sdl = graph.into_sdl().unwrap();
        println!("{}", sdl);

        // Ensure SDL/JSON serialization work as a expected
        let graph = FederatedGraph::from_sdl(&sdl).unwrap();
        let graph = serde_json::from_value(serde_json::to_value(&graph).unwrap()).unwrap();

        let mut federated_graph_config = match self.config_sdl {
            Some(sdl) => {
                parser_sdl::parse(&sdl, &HashMap::new(), &MockConnectorParsers::default())
                    .await
                    .expect("supergraph config SDL to be valid")
                    .federated_graph_config
            }
            None => None,
        }
        .unwrap_or_default();

        federated_graph_config.header_rules.extend(self.header_rules);

        let config = engine_config_builder::build_config(&federated_graph_config, graph).into_latest();

        TestFederationEngine::new(Arc::new(
            engine_v2::Engine::new(
                Arc::new(config.try_into().unwrap()),
                None,
                TestRuntime {
                    fetcher: runtime_local::NativeFetcher::runtime_fetcher(),
                    trusted_documents: self
                        .trusted_documents
                        .map(trusted_documents_client::Client::new)
                        .unwrap_or_else(|| {
                            trusted_documents_client::Client::new(runtime_noop::trusted_documents::NoopTrustedDocuments)
                        }),
                    kv: runtime_local::InMemoryKvStore::runtime(),
                    meter: grafbase_tracing::metrics::meter_from_global_provider(),
                    hooks: self.hooks,
                },
            )
            .await,
        ))
    }
}

#[async_trait::async_trait]
impl SchemaSource for String {
    async fn sdl(&self) -> String {
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
    async fn sdl(&self) -> String {
        T::sdl(self).await
    }

    fn url(&self) -> String {
        T::url(self)
    }
}

#[async_trait::async_trait]
impl SchemaSource for MockGraphQlServer {
    async fn sdl(&self) -> String {
        self.schema.sdl()
    }

    fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port())
    }
}
