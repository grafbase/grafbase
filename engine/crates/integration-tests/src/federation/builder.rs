use async_graphql_parser::types::ServiceDocument;
use engine_v2::Engine;

use super::TestFederationEngine;

#[must_use]
pub struct FederationEngineBuilder {
    schemas: Vec<(String, ServiceDocument)>,
}

pub trait EngineV2Ext {
    fn build() -> FederationEngineBuilder {
        FederationEngineBuilder { schemas: vec![] }
    }
}

impl EngineV2Ext for engine_v2::Engine {}

pub trait SchemaSource {
    fn sdl(&self) -> String;
}

impl FederationEngineBuilder {
    pub fn with_schema(mut self, name: &str, schema: impl SchemaSource) -> Self {
        self.schemas.push((
            name.to_string(),
            async_graphql_parser::parse_schema(schema.sdl()).expect("schema to be well formed"),
        ));
        self
    }

    pub fn finish(self) -> TestFederationEngine {
        let mut subgraphs = graphql_composition::Subgraphs::default();
        for (name, schema) in self.schemas {
            subgraphs.ingest(&schema, &name);
        }
        let graph = graphql_composition::compose(&subgraphs)
            .into_result()
            .expect("schemas to compose succesfully");

        TestFederationEngine {
            engine: Engine::new(graph.into()),
        }
    }
}

// At some point we could provide one of these that introspects.  But for now just a dumb string impl
impl SchemaSource for String {
    fn sdl(&self) -> String {
        self.clone()
    }
}

impl<T> SchemaSource for T
where
    T: crate::mocks::graphql::Schema,
{
    fn sdl(&self) -> String {
        crate::mocks::graphql::Schema::sdl(self)
    }
}
