use std::{collections::HashMap, str::FromStr, sync::Arc};

use super::Inner;
use engine::{registry::resolvers::graphql::QueryBatcher, Schema};
use futures::future::{join_all, BoxFuture};
use parser_sdl::{ConnectorParsers, GraphqlDirective, OpenApiDirective, ParseResult, PostgresDirective, Registry};
use postgres_connector_types::transport::DirectTcpTransport;
use runtime::udf::{CustomResolverInvoker, CustomResolverRequestPayload, UdfInvokerInner};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::Engine;

#[must_use]
pub struct EngineBuilder {
    schema: String,
    openapi_specs: HashMap<String, String>,
    environment_variables: HashMap<String, String>,
    custom_resolvers: Option<CustomResolverInvoker>,
}

pub struct RequestContext {
    pub ray_id: String,
    pub headers: http::HeaderMap,
    pub wait_until: UnboundedSender<BoxFuture<'static, ()>>,
}

impl RequestContext {
    pub fn new(headers: HashMap<String, String>) -> (Self, UnboundedReceiver<BoxFuture<'static, ()>>) {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        let ctx = RequestContext {
            ray_id: ulid::Ulid::new().to_string(),
            headers: http::HeaderMap::from_iter(headers.iter().map(|(k, v)| {
                (
                    http::HeaderName::from_str(k).unwrap(),
                    http::HeaderValue::from_str(v).unwrap(),
                )
            })),
            wait_until: sender,
        };
        (ctx, receiver)
    }

    pub async fn wait_for_all(mut receiver: UnboundedReceiver<BoxFuture<'static, ()>>) {
        // Wait simultaneously on everything immediately accessible
        join_all(std::iter::from_fn(|| receiver.try_recv().ok())).await;
        // Wait sequentially on the rest
        while let Some(fut) = receiver.recv().await {
            fut.await;
        }
    }
}

#[async_trait::async_trait]
impl runtime::context::RequestContext for RequestContext {
    fn ray_id(&self) -> &str {
        &self.ray_id
    }

    async fn wait_until(&self, fut: BoxFuture<'static, ()>) {
        self.wait_until.send(fut).unwrap();
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

    pub fn with_custom_resolvers(
        self,
        invoker: impl UdfInvokerInner<CustomResolverRequestPayload> + Send + Sync + 'static,
    ) -> Self {
        Self {
            custom_resolvers: Some(CustomResolverInvoker::new(invoker)),
            ..self
        }
    }

    pub async fn build(self) -> Engine {
        let ParseResult {
            mut registry,
            global_cache_rules,
            ..
        } = parser_sdl::parse(&self.schema, &self.environment_variables, &self)
            .await
            .unwrap();

        global_cache_rules.apply(&mut registry).unwrap();

        let registry: Registry = serde_json::from_value(serde_json::to_value(registry).unwrap()).unwrap();

        let postgres = {
            let mut transports = HashMap::new();
            for (name, definition) in &registry.postgres_databases {
                let transport = DirectTcpTransport::new(definition.connection_string()).await.unwrap();
                transports.insert(name.to_string(), transport);
            }
            runtime::pg::PgTransportFactory::new(Box::new(runtime_local::LocalPgTransportFactory::new(transports)))
        };

        // engine-v2 tests don't use wait_until so it's not a problem for the receiver to be
        // dropped immediately.
        let (sender, _) = tokio::sync::mpsc::unbounded_channel();
        let mut schema_builder = Schema::build(registry)
            .data(QueryBatcher::new())
            .data(runtime::Context::new(
                &Arc::new(RequestContext {
                    ray_id: String::new(),
                    headers: Default::default(),
                    wait_until: sender,
                }),
                runtime::context::LogContext {
                    fetch_log_endpoint_url: None,
                    request_log_event_id: None,
                },
            ))
            .data(postgres);

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
            directive
                .transforms
                .as_ref()
                .and_then(|transforms| transforms.prefix_types.as_deref()),
        )
        .await
        .map_err(|errors| errors.into_iter().map(|error| error.to_string()).collect::<Vec<_>>())
    }

    async fn fetch_and_parse_postgres(&self, directive: &PostgresDirective) -> Result<Registry, Vec<String>> {
        let transport = DirectTcpTransport::new(directive.connection_string())
            .await
            .map_err(|error| vec![error.to_string()])?;

        parser_postgres::introspect(&transport, directive.name(), directive.namespace())
            .await
            .map_err(|error| vec![error.to_string()])
    }
}
