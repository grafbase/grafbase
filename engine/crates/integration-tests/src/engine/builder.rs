use super::{dynamo::enable_local_dynamo, Inner};
use crate::Engine;
use engine::{registry::resolvers::graphql::QueryBatcher, Schema};
use futures::future::BoxFuture;
use parser_sdl::{ConnectorParsers, GraphqlDirective, OpenApiDirective, ParseResult, PostgresDirective, Registry};
use postgres_types::transport::TcpTransport;
use runtime::udf::{CustomResolverRequestPayload, CustomResolversEngine, UdfInvoker};
use std::{collections::HashMap, sync::Arc};

#[must_use]
pub struct EngineBuilder {
    schema: String,
    openapi_specs: HashMap<String, String>,
    environment_variables: HashMap<String, String>,
    custom_resolvers: Option<CustomResolversEngine>,
    local_dynamo: bool,
}

struct RequestContext {
    ray_id: String,
    headers: http::HeaderMap,
}

#[async_trait::async_trait]
impl runtime::context::RequestContext for RequestContext {
    fn ray_id(&self) -> &str {
        &self.ray_id
    }

    async fn wait_until(&self, _fut: BoxFuture<'static, ()>) {
        unimplemented!("wait_until not implemented for integration tests yet...");
    }

    fn headers(&self) -> &http::HeaderMap {
        &self.headers
    }
}

impl EngineBuilder {
    pub fn new(schema: impl Into<String>) -> Self {
        EngineBuilder {
            schema: schema.into(),
            openapi_specs: HashMap::new(),
            environment_variables: HashMap::new(),
            custom_resolvers: None,
            local_dynamo: false,
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

    pub fn with_local_dynamo(mut self) -> Self {
        self.local_dynamo = true;
        self
    }

    pub fn with_custom_resolvers(
        self,
        invoker: impl UdfInvoker<CustomResolverRequestPayload> + Send + Sync + 'static,
    ) -> Self {
        Self {
            custom_resolvers: Some(CustomResolversEngine::new(Box::new(invoker))),
            ..self
        }
    }

    pub async fn build(self) -> Engine {
        let ParseResult { registry, .. } = parser_sdl::parse(&self.schema, &self.environment_variables, true, &self)
            .await
            .unwrap();

        let registry = serde_json::from_value(serde_json::to_value(registry).unwrap()).unwrap();

        let mut schema_builder = Schema::build(registry)
            .data(QueryBatcher::new())
            .data(runtime::Context::new(
                &Arc::new(RequestContext {
                    ray_id: String::new(),
                    headers: Default::default(),
                }),
                runtime::context::LogContext {
                    fetch_log_endpoint_url: None,
                    request_log_event_id: None,
                },
            ));

        if self.local_dynamo {
            schema_builder = enable_local_dynamo(schema_builder).await;
        }

        if let Some(custom_resolvers) = self.custom_resolvers {
            schema_builder = schema_builder.data(custom_resolvers);
        }

        let schema = schema_builder.finish();

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

    async fn fetch_and_parse_graphql(&self, directive: GraphqlDirective) -> Result<Registry, Vec<String>> {
        parser_graphql::parse_schema(
            reqwest::Client::new(),
            &directive.name,
            directive.namespace,
            &directive.url,
            directive.headers(),
            directive.introspection_headers(),
        )
        .await
        .map_err(|errors| errors.into_iter().map(|error| error.to_string()).collect::<Vec<_>>())
    }

    async fn fetch_and_parse_postgres(&self, directive: &PostgresDirective) -> Result<Registry, Vec<String>> {
        let transport = TcpTransport::new(directive.connection_string())
            .await
            .map_err(|error| vec![error.to_string()])?;

        parser_postgres::introspect(&transport, directive.name(), directive.namespace())
            .await
            .map_err(|error| vec![error.to_string()])
    }
}
