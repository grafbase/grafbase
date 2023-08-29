use std::{collections::HashMap, sync::Arc};

use grafbase_engine::{registry::resolvers::graphql::QueryBatcher, Schema};
use grafbase_runtime::GraphqlRequestExecutionContext;
use sdl_parser::{ConnectorParsers, GraphqlDirective, OpenApiDirective, ParseResult, Registry};

use crate::Engine;

use super::Inner;

#[must_use]
pub struct EngineBuilder {
    schema: String,
    openapi_specs: HashMap<String, String>,
    environment_variables: HashMap<String, String>,
}

impl EngineBuilder {
    pub fn new(schema: String) -> Self {
        EngineBuilder {
            schema,
            openapi_specs: HashMap::new(),
            environment_variables: HashMap::new(),
        }
    }

    pub fn with_openapi_schema(mut self, url: impl Into<String>, spec: impl Into<String>) -> Self {
        self.openapi_specs.insert(url.into(), spec.into());
        self
    }

    pub fn with_env_var(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.environment_variables.insert(name.into(), value.into());
        self
    }

    pub async fn build(self) -> Engine {
        let ParseResult { registry, .. } = sdl_parser::parse(&self.schema, &self.environment_variables, &self)
            .await
            .unwrap();

        let registry = serde_json::from_value(serde_json::to_value(registry).unwrap()).unwrap();

        let schema = Schema::build(registry)
            .data(QueryBatcher::new())
            .data(GraphqlRequestExecutionContext {
                ray_id: String::new(),
                headers: Default::default(),
            })
            .finish();

        Engine {
            inner: Arc::new(Inner { schema }),
        }
    }
}

#[async_trait::async_trait]
impl ConnectorParsers for EngineBuilder {
    async fn fetch_and_parse_openapi(&self, directive: OpenApiDirective) -> Result<Registry, Vec<String>> {
        let url = directive.schema_url.clone();

        let spec = self
            .openapi_specs
            .get(&url)
            .unwrap_or_else(|| panic!("tried to test with an unexpected openapi url: {url}"));

        let mut registry = Registry::new();

        parser_openapi::parse_spec(
            spec.clone(),
            parser_openapi::Format::guess(None, &url),
            directive.into(),
            &mut registry,
        )
        .map_err(|errors| errors.into_iter().map(|error| error.to_string()).collect::<Vec<_>>())?;

        Ok(registry)
    }

    async fn fetch_and_parse_graphql(&self, _directive: GraphqlDirective) -> Result<Registry, Vec<String>> {
        todo!("someone should implement this sometime, similar to the above")
    }
}
