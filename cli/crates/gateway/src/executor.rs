use std::{collections::HashMap, sync::Arc};

use common_types::auth::ExecutionAuth;
use engine::{registry::resolvers::graphql, RequestHeaders};
use gateway_core::{RequestContext, StreamingFormat};
use graphql_extensions::{authorization::AuthExtension, runtime_log::RuntimeLogExtension};
use postgres_connector_types::transport::DirectTcpTransport;
use runtime::pg::PgTransportFactory;
use runtime_local::{Bridge, LocalPgTransportFactory, UdfInvokerImpl};

pub struct Executor {
    #[allow(dead_code)]
    env_vars: HashMap<String, String>,
    bridge: Bridge,
    registry: Arc<engine::Registry>,
    postgres: LocalPgTransportFactory,
}

impl Executor {
    pub(crate) async fn new(
        env_vars: HashMap<String, String>,
        bridge: Bridge,
        registry: Arc<engine::Registry>,
    ) -> Result<Self, crate::Error> {
        let postgres = {
            let mut transports = HashMap::new();
            for (name, definition) in &registry.postgres_databases {
                let transport = DirectTcpTransport::new(definition.connection_string())
                    .await
                    .map_err(|error| crate::Error::Internal(error.to_string()))?;

                transports.insert(name.to_string(), transport);
            }
            LocalPgTransportFactory::new(transports)
        };

        Ok(Self {
            env_vars,
            bridge,
            registry,
            postgres,
        })
    }

    async fn build_schema(
        &self,
        ctx: &Arc<crate::Context>,
        auth: ExecutionAuth,
    ) -> Result<engine::Schema, crate::Error> {
        let runtime_ctx = runtime::context::Context::new(
            ctx,
            runtime::context::LogContext {
                fetch_log_endpoint_url: Some(format!(
                    "http://{}:{}",
                    std::net::Ipv4Addr::LOCALHOST,
                    self.bridge.port()
                )),
                request_log_event_id: None,
            },
        );

        let resolver_engine = UdfInvokerImpl::custom_resolver(self.bridge.clone());

        Ok(engine::Schema::build(engine::Registry::clone(&self.registry))
            .data(engine::TraceId(ctx.ray_id().to_string()))
            .data(graphql::QueryBatcher::new())
            .data(resolver_engine)
            .data(auth)
            .data(PgTransportFactory::new(Box::new(self.postgres.clone())))
            .data(RequestHeaders::from(&ctx.headers_as_map()))
            .data(runtime_ctx)
            .extension(RuntimeLogExtension::new(Box::new(
                runtime_local::LogEventReceiverImpl::new(self.bridge.clone()),
            )))
            .extension(AuthExtension::new(ctx.ray_id().to_string()))
            .finish())
    }
}

#[async_trait::async_trait]
impl gateway_core::Executor for Executor {
    type Context = crate::Context;
    type Error = crate::Error;
    type StreamingResponse = crate::Response;

    async fn execute(
        self: Arc<Self>,
        ctx: Arc<crate::Context>,
        auth: ExecutionAuth,
        request: engine::Request,
    ) -> Result<engine::Response, crate::Error> {
        let schema = self.build_schema(&ctx, auth).await?;
        Ok(schema.execute(request).await)
    }

    async fn execute_stream(
        self: Arc<Self>,
        ctx: Arc<crate::Context>,
        auth: ExecutionAuth,
        request: engine::Request,
        streaming_format: StreamingFormat,
    ) -> Result<Self::StreamingResponse, crate::Error> {
        use axum::response::IntoResponse;
        let schema = self.build_schema(&ctx, auth).await?;
        let payload_stream = Box::pin(schema.execute_stream(request));
        let (headers, bytes_stream) = gateway_core::encode_stream_response(payload_stream, streaming_format);

        Ok((headers, axum::body::Body::from_stream(bytes_stream))
            .into_response()
            .into())
    }
}
