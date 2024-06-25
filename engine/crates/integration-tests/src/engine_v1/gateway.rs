#![allow(dead_code)] // Can remove this when this code is being used.

use std::{collections::HashMap, future::IntoFuture, str::FromStr, sync::Arc};

use engine::{AuthConfig, Variables};
use futures::future::BoxFuture;
use gateway_core::{AuthService, CacheConfig, ConstructableResponse, ExecutionAuth, RequestContext, StreamingFormat};
use http::HeaderMap;
use registry_for_cache::PartialCacheRegistry;
use registry_v2::rate_limiting::RateLimitConfig;
use runtime::{kv::KvStore, trusted_documents_client, udf::UdfInvoker};
use runtime_noop::kv::NoopKvStore;

use crate::{mock_trusted_documents::MockTrustedDocumentsClient, udfs::RustUdfs, TestTrustedDocument};

use super::GraphQlRequest;

pub struct GatewayBuilder {
    engine: Arc<super::Engine>,
    partial_cache_registry: PartialCacheRegistry,
    trusted_documents: Option<MockTrustedDocumentsClient>,
    auth_config: AuthConfig,
    authorizers: Option<RustUdfs>,
    rate_limiting_config: RateLimitConfig,
    auth_service: Option<AuthService>,
}

impl GatewayBuilder {
    pub(super) fn new(engine: super::Engine, partial_cache_registry: PartialCacheRegistry) -> Self {
        Self {
            engine: Arc::new(engine),
            partial_cache_registry,
            trusted_documents: None,
            auth_config: Default::default(),
            authorizers: None,
            rate_limiting_config: Default::default(),
            auth_service: None,
        }
    }

    pub fn with_authorizers(self, authorizers: RustUdfs) -> Self {
        Self {
            authorizers: Some(authorizers),
            ..self
        }
    }

    pub fn with_auth(self, auth_config: AuthConfig) -> Self {
        Self { auth_config, ..self }
    }

    pub fn with_trusted_documents(mut self, branch_id: String, documents: Vec<TestTrustedDocument>) -> Self {
        self.trusted_documents = Some(MockTrustedDocumentsClient {
            _branch_id: branch_id,
            documents,
        });
        self
    }

    pub fn with_rate_limiting_config(mut self, config: RateLimitConfig) -> Self {
        self.rate_limiting_config = config;
        self
    }

    pub fn with_auth_service(mut self, auth_service: AuthService) -> Self {
        self.auth_service = Some(auth_service);
        self
    }

    pub fn build(self) -> GatewayTester {
        let cache = runtime_local::InMemoryCache::runtime(runtime::cache::GlobalCacheConfig {
            enabled: true,
            ..Default::default()
        });

        let cache_config = CacheConfig {
            global_enabled: true,
            subdomain: "integration-test".to_string(),
            host_name: "integration-test".to_string(),
            partial_registry: Arc::new(self.partial_cache_registry),
            common_cache_tags: vec![],
        };

        // This AuthService is used to authenticate "user" requests
        let auth = self.auth_service.unwrap_or(AuthService::new_v1(
            self.auth_config,
            KvStore::new(NoopKvStore),
            UdfInvoker::new(self.authorizers.unwrap_or_default()),
            "my-identity-is-ray".into(),
        ));

        // This authorizor is used to authorize admin requests.
        // Not to be confused with the authorizors that live inside AuthService above :|
        let authorizer = Box::new(AnythingGoes);

        let trusted_documents = trusted_documents_client::Client::new(self.trusted_documents.unwrap_or_default());

        let rate_limiting = runtime_local::InMemoryRateLimiting::new(&self.rate_limiting_config.rules);

        GatewayTester {
            inner: Arc::new(gateway_core::Gateway::new(
                self.engine,
                cache,
                cache_config,
                auth,
                authorizer,
                trusted_documents,
                grafbase_tracing::metrics::meter_from_global_provider(),
                Box::new(rate_limiting),
            )),
        }
    }
}

struct AnythingGoes;

#[async_trait::async_trait]
impl gateway_core::Authorizer for AnythingGoes {
    type Context = GatewayTesterRequestContext;
    async fn authorize_admin_request(
        &self,
        _ctx: &Arc<Self::Context>,
        _request: &async_graphql::Request,
    ) -> Result<(), gateway_core::AdminAuthError> {
        Ok(())
    }
}

pub struct GatewayTesterRequestContext {
    headers: http::HeaderMap,
}

#[async_trait::async_trait]
impl RequestContext for GatewayTesterRequestContext {
    fn ray_id(&self) -> &str {
        "what-do-you-mean-im-not-a-ray-id-how-very-dare-you"
    }

    async fn wait_until(&self, _fut: BoxFuture<'static, ()>) {
        todo!("probably want to implement this")
    }

    fn headers(&self) -> &http::HeaderMap {
        &self.headers
    }
}

/// A wrapper around gateway_core::Gateway that makes it slightly easier to use in tests
#[derive(Clone)]
pub struct GatewayTester {
    inner: Arc<gateway_core::Gateway<super::Engine>>,
}

impl GatewayTester {
    pub fn execute(&self, operation: impl Into<GraphQlRequest>) -> GatewayTesterExecutionRequest {
        GatewayTesterExecutionRequest {
            graphql: operation.into(),
            headers: HashMap::new(),
            gateway: self.inner.clone(),
        }
    }
}

#[must_use]
pub struct GatewayTesterExecutionRequest {
    graphql: GraphQlRequest,
    headers: HashMap<String, String>,
    gateway: Arc<gateway_core::Gateway<super::Engine>>,
}

impl GatewayTesterExecutionRequest {
    /// Adds a header into the request
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    pub fn variables(mut self, variables: impl serde::Serialize) -> Self {
        self.graphql.variables = Some(Variables::from_json(
            serde_json::to_value(variables).expect("variables to be serializable"),
        ));
        self
    }
}

impl IntoFuture for GatewayTesterExecutionRequest {
    type Output = Result<(Arc<engine::Response>, HeaderMap), Error>;

    type IntoFuture = BoxFuture<'static, Result<(Arc<engine::Response>, HeaderMap), Error>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let request = self.graphql.into_engine_request();
            // .data(RequestHeaders::from(&self.headers));

            // TODO: Do something with headers...
            self.gateway
                .execute(
                    &Arc::new(GatewayTesterRequestContext {
                        headers: self
                            .headers
                            .into_iter()
                            .map(|(k, v)| {
                                (
                                    http::HeaderName::from_str(&k).expect("valid header name"),
                                    http::HeaderValue::from_str(&v).expect("valid header value"),
                                )
                            })
                            .collect(),
                    }),
                    request,
                )
                .await
            // TODO: Probably want to do any wait_untils here as well...
        })
    }
}

#[async_trait::async_trait]
impl gateway_core::Executor for super::Engine {
    type Error = Error;

    type Context = GatewayTesterRequestContext;

    type StreamingResponse = UnconstructableResponse;

    async fn execute(
        self: Arc<Self>,
        _ctx: Arc<Self::Context>,
        _auth: ExecutionAuth,
        request: engine::Request,
    ) -> Result<engine::Response, Self::Error> {
        Ok(self.inner.schema.execute(request).await)
    }

    async fn execute_stream(
        self: Arc<Self>,
        _ctx: Arc<Self::Context>,
        _auth: ExecutionAuth,
        _request: engine::Request,
        _streaming_format: StreamingFormat,
    ) -> Result<Self::StreamingResponse, Self::Error> {
        unimplemented!("implement execute_stream if you want to test that, I do not right now")
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error(transparent)]
    Cache(runtime::cache::Error),
    #[error(transparent)]
    Ratelimit(#[from] runtime::rate_limiting::Error),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<gateway_core::Error> for Error {
    fn from(err: gateway_core::Error) -> Self {
        match err {
            gateway_core::Error::BadRequest(msg) => Self::BadRequest(msg),
            gateway_core::Error::Cache(err) => Self::Cache(err),
            gateway_core::Error::Serialization(msg) => Self::Serialization(msg),
            gateway_core::Error::Ratelimit(err) => Self::Ratelimit(err),
        }
    }
}

// I don't want to implement this properly just now.  If you need this, I suggest you fix it up
pub struct UnconstructableResponse;

#[allow(unused_variables)]
impl ConstructableResponse for UnconstructableResponse {
    type Error = Error;

    fn error(code: http::StatusCode, message: &str) -> Self {
        unimplemented!("if you want me, implement me")
    }

    fn engine(response: Arc<engine::Response>, headers: http::HeaderMap) -> Result<Self, Self::Error> {
        unimplemented!("if you want me, implement me")
    }

    fn admin(response: async_graphql::Response) -> Result<Self, Self::Error> {
        unimplemented!("if you want me, implement me")
    }
}
