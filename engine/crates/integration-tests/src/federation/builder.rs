use std::collections::HashMap;

use async_graphql_parser::types::ServiceDocument;
use engine_v2::Engine;
use parser_sdl::connector_parsers::MockConnectorParsers;

use crate::MockGraphQlServer;

use super::TestFederationEngine;

#[must_use]
pub struct FederationEngineBuilder {
    schemas: Vec<(String, String, ServiceDocument)>,
    config_sdl: Option<String>,
}

pub trait EngineV2Ext {
    fn builder() -> FederationEngineBuilder {
        FederationEngineBuilder {
            schemas: vec![],
            config_sdl: None,
        }
    }
}

impl EngineV2Ext for engine_v2::Engine {}

#[async_trait::async_trait]
pub trait SchemaSource {
    async fn sdl(&self) -> String;
    fn url(&self) -> String;
}

impl FederationEngineBuilder {
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

    pub async fn finish(self) -> TestFederationEngine {
        let mut subgraphs = graphql_composition::Subgraphs::default();
        for (name, url, schema) in self.schemas {
            subgraphs.ingest(&schema, &name, &url);
        }
        let graph = graphql_composition::compose(&subgraphs)
            .into_result()
            .expect("schemas to compose succesfully");

        let federated_graph_config = match self.config_sdl {
            Some(sdl) => {
                parser_sdl::parse(&sdl, &HashMap::new(), false, &MockConnectorParsers::default())
                    .await
                    .expect("supergraph config SDL to be valid")
                    .federated_graph_config
            }
            None => None,
        }
        .unwrap_or_default();

        let config = engine_config_builder::build_config(&federated_graph_config, graph).into_latest();

        TestFederationEngine {
            engine: Engine::build(
                config.into(),
                engine_v2::EngineRuntime {
                    fetcher: runtime_local::NativeFetcher::runtime_fetcher(),
                    kv: runtime_local::InMemoryKvStore::runtime_kv(),
                },
            ),
        }
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
