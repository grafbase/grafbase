use async_graphql_parser::types::ServiceDocument;
use engine_v2::Engine;
use graphql_composition::FederatedGraph;

use crate::MockGraphQlServer;

use super::TestFederationEngine;

#[must_use]
pub struct FederationEngineBuilder {
    schemas: Vec<(String, String, ServiceDocument)>,
}

pub trait EngineV2Ext {
    fn build() -> FederationEngineBuilder {
        FederationEngineBuilder { schemas: vec![] }
    }
}

impl EngineV2Ext for engine_v2::Engine {}

#[async_trait::async_trait]
pub trait SchemaSource {
    async fn sdl(&self) -> String;
    fn url(&self) -> String;
}

impl FederationEngineBuilder {
    pub async fn with_schema(mut self, name: &str, schema: &impl SchemaSource) -> Self {
        self.schemas.push((
            name.to_string(),
            schema.url(),
            async_graphql_parser::parse_schema(schema.sdl().await).expect("schema to be well formed"),
        ));
        self
    }

    pub fn finish(self) -> TestFederationEngine {
        let mut subgraphs = graphql_composition::Subgraphs::default();
        for (name, url, schema) in self.schemas {
            subgraphs.ingest(&schema, &name, &url);
        }
        let graph = graphql_composition::compose(&subgraphs)
            .into_result()
            .expect("schemas to compose succesfully");

        let FederatedGraph::V1(graph) = graph;
        let config = engine_v2::VersionedConfig::V1(graph).into_latest();

        TestFederationEngine {
            engine: Engine::new(
                config.into(),
                engine_v2::EngineRuntime {
                    fetcher: runtime_local::NativeFetcher::runtime_fetcher(),
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
