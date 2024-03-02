use auth::AnyApiKeyProvider;
use engine::{EngineV1, EngineV1Env, HttpGraphqlRequest, HttpGraphqlResponse, RequestHeaders, SchemaVersion};
use gateway_v2_auth::AuthService;
use graphql_extensions::{authorization::AuthExtension, runtime_log::RuntimeLogExtension};
use postgres_connector_types::transport::DirectTcpTransport;
use runtime::{auth::AccessToken, pg::PgTransportFactory};
use runtime_local::{InMemoryCache, InMemoryKvStore, LocalPgTransportFactory, UdfInvokerImpl};
use std::{collections::HashMap, sync::Arc};

mod auth;
mod serving;

pub use runtime_local::Bridge;

#[derive(Clone)]
pub struct Gateway(Arc<GatewayInner>);

impl std::ops::Deref for Gateway {
    type Target = GatewayInner;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

pub struct GatewayInner {
    registry: Arc<engine::Registry>,
    bridge: Bridge,
    postgres: LocalPgTransportFactory,
    auth: AuthService,
    schema_version: SchemaVersion,
    env: EngineV1Env,
}

impl Gateway {
    pub async fn new(
        _env_vars: HashMap<String, String>,
        bridge: Bridge,
        registry: Arc<engine::Registry>,
    ) -> Result<Self, String> {
        let auth = gateway_v2_auth::AuthService::new_v1(
            registry.auth.clone(),
            InMemoryKvStore::runtime(),
            runtime_local::UdfInvokerImpl::authorizer(bridge.clone()),
            String::new(),
        )
        .with_first_authorizer(AnyApiKeyProvider);

        let postgres = {
            let mut transports = HashMap::new();
            for (name, definition) in &registry.postgres_databases {
                let transport = DirectTcpTransport::new(definition.connection_string())
                    .await
                    .map_err(|error| error.to_string())?;

                transports.insert(name.to_string(), transport);
            }
            LocalPgTransportFactory::new(transports)
        };

        let async_runtime = runtime_local::TokioCurrentRuntime::runtime();
        Ok(Self(Arc::new(GatewayInner {
            auth,
            registry,
            bridge,
            postgres,
            env: EngineV1Env {
                cache: InMemoryCache::runtime(async_runtime.clone()),
                cache_operation_cache_control: false,
                async_runtime,
            },
            schema_version: SchemaVersion::from(ulid::Ulid::new().to_string()),
        })))
    }

    pub fn into_router(self) -> axum::Router {
        serving::router(self)
    }
}

impl GatewayInner {
    async fn execute(&self, headers: &http::HeaderMap, request: HttpGraphqlRequest<'_>) -> HttpGraphqlResponse {
        let ray_id = ulid::Ulid::new().to_string();

        let Some(AccessToken::V1(auth)) = self.auth.authorize(headers).await else {
            return HttpGraphqlResponse::unauthorized();
        };

        let schema = engine::Schema::build(engine::Registry::clone(&self.registry))
            .data(engine::TraceId(ray_id.to_string()))
            .data(engine::registry::resolvers::graphql::QueryBatcher::new())
            .data(UdfInvokerImpl::custom_resolver(self.bridge.clone()))
            .data(auth.clone())
            .data(PgTransportFactory::new(Box::new(self.postgres.clone())))
            .data(RequestHeaders::new(headers.iter().filter_map(|(name, value)| {
                Some((name.to_string(), value.to_str().ok()?.to_string()))
            })))
            .data(engine::LogContext {
                fetch_log_endpoint_url: Some(format!(
                    "http://{}:{}",
                    std::net::Ipv4Addr::LOCALHOST,
                    self.bridge.port()
                )),
                request_log_event_id: None,
            })
            .extension(RuntimeLogExtension::new(Box::new(
                runtime_local::LogEventReceiverImpl::new(self.bridge.clone()),
            )))
            .extension(AuthExtension::new(ray_id.to_string()))
            .finish();

        let engine = EngineV1::new(schema, self.schema_version.clone(), self.env.clone());

        engine
            .execute_with_access_token(headers, &AccessToken::V1(auth), &ray_id, request)
            .await
    }
}
