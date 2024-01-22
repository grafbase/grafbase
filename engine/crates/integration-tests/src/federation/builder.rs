use std::{collections::HashMap, sync::Arc};

use async_graphql_parser::types::ServiceDocument;
use gateway_v2::Gateway;
use graphql_mocks::MockGraphQlServer;
use parser_sdl::connector_parsers::MockConnectorParsers;

use super::TestFederationGateway;

#[must_use]
pub struct FederationGatewayBuilder {
    schemas: Vec<(String, String, ServiceDocument)>,
    config_sdl: Option<String>,
}

pub trait GatewayV2Ext {
    fn builder() -> FederationGatewayBuilder {
        FederationGatewayBuilder {
            schemas: vec![],
            config_sdl: None,
        }
    }
}

impl GatewayV2Ext for gateway_v2::Gateway {}

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

    pub async fn finish(self) -> TestFederationGateway {
        let mut subgraphs = graphql_composition::Subgraphs::default();
        for (name, url, schema) in self.schemas {
            subgraphs.ingest(&schema, &name, &url);
        }
        let graph = graphql_composition::compose(&subgraphs)
            .into_result()
            .expect("schemas to compose succesfully");
        println!("{}", graph.to_sdl().unwrap());
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

        TestFederationGateway {
            gateway: Arc::new(Gateway::new(
                config.into(),
                engine_v2::EngineEnv {
                    fetcher: runtime_local::NativeFetcher::runtime_fetcher(),
                },
                gateway_v2::GatewayEnv {
                    kv: runtime_local::InMemoryKvStore::runtime(),
                    cache: runtime_local::InMemoryCache::runtime(runtime::cache::GlobalCacheConfig {
                        enabled: true,
                        ..Default::default()
                    }),
                },
            )),
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
